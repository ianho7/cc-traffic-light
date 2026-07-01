param(
    [string]$OutDir = "$env:TEMP\cc-traffic-light-codex-notify-probe",
    [string]$ForwardJsonBase64 = ""
)

$ErrorActionPreference = "Stop"

function Read-StandardInputBytes {
    $stream = [Console]::OpenStandardInput()
    $buffer = New-Object byte[] 8192
    $memory = New-Object System.IO.MemoryStream
    while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
        $memory.Write($buffer, 0, $read)
    }
    $memory.ToArray()
}

function Decode-InputBytes {
    param([byte[]]$Bytes)

    if ($Bytes.Length -eq 0) {
        return @{
            Encoding = "empty"
            Text = ""
        }
    }

    if ($Bytes.Length -ge 3 -and $Bytes[0] -eq 0xEF -and $Bytes[1] -eq 0xBB -and $Bytes[2] -eq 0xBF) {
        return @{
            Encoding = "utf8_bom"
            Text = [System.Text.Encoding]::UTF8.GetString($Bytes, 3, $Bytes.Length - 3)
        }
    }

    if ($Bytes.Length -ge 2 -and $Bytes[0] -eq 0xFF -and $Bytes[1] -eq 0xFE) {
        return @{
            Encoding = "utf16le_bom"
            Text = [System.Text.Encoding]::Unicode.GetString($Bytes, 2, $Bytes.Length - 2)
        }
    }

    $looksUtf16Le = $false
    for ($i = 1; $i -lt $Bytes.Length; $i += 2) {
        if ($Bytes[$i] -eq 0) {
            $looksUtf16Le = $true
            break
        }
    }

    if ($looksUtf16Le) {
        return @{
            Encoding = "utf16le"
            Text = [System.Text.Encoding]::Unicode.GetString($Bytes)
        }
    }

    return @{
        Encoding = "utf8"
        Text = [System.Text.Encoding]::UTF8.GetString($Bytes)
    }
}

function ConvertTo-RedactedShape {
    param($Value)

    if ($null -eq $Value) {
        return @{ type = "null" }
    }

    if ($Value -is [System.Array]) {
        $itemShape = "empty_array"
        if ($Value.Count -gt 0) {
            $itemShape = ConvertTo-RedactedShape $Value[0]
        }
        return @{
            type = "array"
            item = $itemShape
        }
    }

    if ($Value -is [System.Management.Automation.PSCustomObject]) {
        $shape = [ordered]@{}
        foreach ($property in $Value.PSObject.Properties) {
            $shape[$property.Name] = ConvertTo-RedactedShape $property.Value
        }
        return $shape
    }

    if ($Value -is [bool]) {
        return @{
            type = "boolean"
            value = "<redacted>"
        }
    }

    if ($Value -is [int] -or $Value -is [long] -or $Value -is [double] -or $Value -is [decimal]) {
        return @{
            type = "number"
            value = "<redacted>"
        }
    }

    return @{
        type = "string"
        value = "<redacted>"
    }
}

function Find-CandidatePaths {
    param(
        $Value,
        [string[]]$Keys,
        [string]$Prefix = '$'
    )

    $paths = @()
    if ($Value -is [System.Management.Automation.PSCustomObject]) {
        foreach ($property in $Value.PSObject.Properties) {
            $next = "$Prefix.$($property.Name)"
            if ($Keys -contains $property.Name) {
                $paths += $next
            }
            $paths += Find-CandidatePaths -Value $property.Value -Keys $Keys -Prefix $next
        }
    } elseif ($Value -is [System.Array]) {
        for ($i = 0; $i -lt $Value.Count; $i++) {
            $paths += Find-CandidatePaths -Value $Value[$i] -Keys $Keys -Prefix "$Prefix[$i]"
        }
    }
    $paths
}

function New-ArgShape {
    param([string[]]$Args)

    $items = @()
    for ($i = 0; $i -lt $Args.Count; $i++) {
        $items += @{
            index = $i
            kind = "string"
            value = "<redacted>"
        }
    }

    @{
        count = $Args.Count
        items = $items
    }
}

function New-ArgShapeFromCount {
    param([int]$Count)

    $items = @()
    for ($i = 0; $i -lt $Count; $i++) {
        $items += @{
            index = $i
            kind = "string"
            value = "<redacted>"
        }
    }

    @{
        count = $Count
        items = $items
    }
}

function Get-ForwardConfig {
    param([string]$JsonBase64)

    if ([string]::IsNullOrWhiteSpace($JsonBase64)) {
        return [pscustomobject]@{
            OriginalCommand = $null
            OriginalArguments = ""
            OriginalArgCount = 0
            TokenCount = 0
            DecodedLength = 0
        }
    }

    $Json = [System.Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($JsonBase64))
    $items = @()
    foreach ($match in [regex]::Matches($Json, '"((?:\\.|[^"\\])*)"')) {
        $items += [regex]::Unescape($match.Groups[1].Value)
    }
    if ($items.Count -eq 0) {
        return [pscustomobject]@{
            OriginalCommand = $null
            OriginalArguments = ""
            OriginalArgCount = 0
            TokenCount = 0
            DecodedLength = $Json.Length
        }
    }

    $originalArgs = @()
    for ($i = 1; $i -lt $items.Count; $i++) {
        $originalArgs += [string]$items[$i]
    }

    return [pscustomobject]@{
        OriginalCommand = [string]$items[0]
        OriginalArguments = Join-ProcessArguments $originalArgs
        OriginalArgCount = $originalArgs.Count
        TokenCount = $items.Count
        DecodedLength = $Json.Length
    }
}

function Join-ProcessArguments {
    param([string[]]$Items)

    $quoted = @()
    foreach ($item in $Items) {
        if ($item -match '[\s"]') {
            $quoted += '"' + ($item -replace '\\', '\\' -replace '"', '\"') + '"'
        } else {
            $quoted += $item
        }
    }

    $quoted -join ' '
}

function Invoke-ForwardedNotify {
    param(
        [string]$Command,
        [string]$Arguments,
        [byte[]]$StdinBytes
    )

    if ([string]::IsNullOrWhiteSpace($Command)) {
        return @{
            attempted = $false
            exit_code = 2
            error = "missing original notify command in -ForwardJson"
        }
    }

    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = $Command
    $startInfo.Arguments = $Arguments
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $startInfo

    try {
        [void]$process.Start()
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if ($StdinBytes.Length -gt 0) {
            $process.StandardInput.BaseStream.Write($StdinBytes, 0, $StdinBytes.Length)
        }
        $process.StandardInput.Close()
        $process.WaitForExit()
        [Console]::Out.Write($stdoutTask.Result)
        [Console]::Error.Write($stderrTask.Result)
        return @{
            attempted = $true
            exit_code = $process.ExitCode
            error = $null
        }
    } catch {
        return @{
            attempted = $true
            exit_code = 1
            error = $_.Exception.Message
        }
    } finally {
        $process.Dispose()
    }
}

$stdinBytes = Read-StandardInputBytes
$decoded = Decode-InputBytes $stdinBytes
$split = Get-ForwardConfig $ForwardJsonBase64
$originalCommand = $split.OriginalCommand
$originalArguments = $split.OriginalArguments
$jsonValue = $null
$isJson = $false
$parseError = $null

if (-not [string]::IsNullOrWhiteSpace($decoded.Text)) {
    try {
        $jsonValue = $decoded.Text | ConvertFrom-Json
        $isJson = $true
    } catch {
        $parseError = $_.Exception.Message
    }
}

$candidatePaths = @{
    session = @()
    thread = @()
    turn = @()
    event_order = @()
}

if ($isJson) {
    $candidatePaths.session = @(Find-CandidatePaths -Value $jsonValue -Keys @("session_id", "sessionId", "sessionID"))
    $candidatePaths.thread = @(Find-CandidatePaths -Value $jsonValue -Keys @("thread_id", "threadId", "conversation_id", "conversationId"))
    $candidatePaths.turn = @(Find-CandidatePaths -Value $jsonValue -Keys @("turn_id", "turnId", "turn", "event"))
    $candidatePaths.event_order = @(Find-CandidatePaths -Value $jsonValue -Keys @("event_order", "eventOrder", "timestamp", "created_at", "createdAt", "time"))
}

$probe = [ordered]@{
    captured_at = (Get-Date).ToUniversalTime().ToString("o")
    argv = @{
        forward_json_base64_present = -not [string]::IsNullOrWhiteSpace($ForwardJsonBase64)
    }
    stdin = @{
        present = $stdinBytes.Length -gt 0
        byte_length = $stdinBytes.Length
        encoding = $decoded.Encoding
        is_json = $isJson
        parse_error = if ($isJson) { $null } else { $parseError }
        shape = if ($isJson) { ConvertTo-RedactedShape $jsonValue } else { $null }
        candidate_paths = $candidatePaths
    }
    original_notify = @{
        command = if ($null -eq $originalCommand) { $null } else { "<redacted>" }
        args = New-ArgShapeFromCount -Count $split.OriginalArgCount
        forward_token_count = $split.TokenCount
        forward_json_decoded_length = $split.DecodedLength
    }
    forward = @{
        attempted = $false
        exit_code = $null
        error = $null
    }
}

try {
    New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
    $fileName = "codex-notify-probe-{0}-{1}.json" -f (Get-Date -Format "yyyyMMdd-HHmmss"), $PID
    $probePath = Join-Path $OutDir $fileName
    $probe | ConvertTo-Json -Depth 64 | Set-Content -LiteralPath $probePath -Encoding UTF8
} catch {
    $probe.forward.error = "probe write failed: $($_.Exception.Message)"
}

$forward = Invoke-ForwardedNotify -Command $originalCommand -Arguments $originalArguments -StdinBytes $stdinBytes
$probe.forward = $forward

try {
    if ($probePath) {
        $probe | ConvertTo-Json -Depth 64 | Set-Content -LiteralPath $probePath -Encoding UTF8
    }
} catch {
    # The wrapper must not block the original notify path because logging failed.
}

exit ([int]$forward.exit_code)
