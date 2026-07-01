param(
    [string]$EventName = "unknown",
    [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $OutDir = Join-Path $env:TEMP "cc-traffic-light-codex-lifecycle-hooks"
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$inputStream = [Console]::OpenStandardInput()
$buffer = New-Object System.IO.MemoryStream
$inputStream.CopyTo($buffer)
$bytes = $buffer.ToArray()

function Decode-InputText {
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
            Text = [Text.Encoding]::UTF8.GetString($Bytes, 3, $Bytes.Length - 3)
        }
    }

    if ($Bytes.Length -ge 2 -and $Bytes[0] -eq 0xFF -and $Bytes[1] -eq 0xFE) {
        return @{
            Encoding = "utf16le_bom"
            Text = [Text.Encoding]::Unicode.GetString($Bytes, 2, $Bytes.Length - 2)
        }
    }

    return @{
        Encoding = "utf8"
        Text = [Text.Encoding]::UTF8.GetString($Bytes)
    }
}

function Get-JsonShape {
    param($Value)

    if ($null -eq $Value) {
        return @{ type = "null" }
    }

    if ($Value -is [string]) {
        return @{ type = "string"; value = "<redacted>" }
    }

    if ($Value -is [bool]) {
        return @{ type = "boolean"; value = "<redacted>" }
    }

    if ($Value -is [int] -or $Value -is [long] -or $Value -is [double] -or $Value -is [decimal]) {
        return @{ type = "number"; value = "<redacted>" }
    }

    if ($Value -is [System.Collections.IEnumerable] -and -not ($Value -is [string]) -and -not ($Value -is [pscustomobject])) {
        $items = @($Value)
        if ($items.Count -eq 0) {
            return @{ type = "array"; item = "empty_array" }
        }
        return @{ type = "array"; item = Get-JsonShape $items[0] }
    }

    $objectShape = [ordered]@{}
    foreach ($property in $Value.PSObject.Properties) {
        $objectShape[$property.Name] = Get-JsonShape $property.Value
    }
    return $objectShape
}

function Find-KeyPaths {
    param(
        $Value,
        [string[]]$Keys,
        [string]$Prefix = '$'
    )

    $paths = New-Object System.Collections.Generic.List[string]

    function Visit {
        param($Node, [string]$Path)

        if ($null -eq $Node) {
            return
        }

        if ($Node -is [pscustomobject]) {
            foreach ($property in $Node.PSObject.Properties) {
                $next = "$Path.$($property.Name)"
                if ($Keys -contains $property.Name) {
                    $paths.Add($next)
                }
                Visit $property.Value $next
            }
            return
        }

        if ($Node -is [System.Collections.IEnumerable] -and -not ($Node -is [string])) {
            $index = 0
            foreach ($item in $Node) {
                Visit $item "$Path[$index]"
                $index += 1
            }
        }
    }

    Visit $Value $Prefix
    return @($paths)
}

$decoded = Decode-InputText $bytes
$parseError = $null
$jsonValue = $null
$isJson = $false

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
    turn = @()
    hook_event = @()
    cwd = @()
    model = @()
    event_order = @()
}

if ($isJson) {
    $candidatePaths.session = @(Find-KeyPaths $jsonValue @("session_id", "sessionId", "sessionID"))
    $candidatePaths.turn = @(Find-KeyPaths $jsonValue @("turn_id", "turnId", "turnID"))
    $candidatePaths.hook_event = @(Find-KeyPaths $jsonValue @("hook_event_name", "hookName", "eventName", "hook_name"))
    $candidatePaths.cwd = @(Find-KeyPaths $jsonValue @("cwd"))
    $candidatePaths.model = @(Find-KeyPaths $jsonValue @("model"))
    $candidatePaths.event_order = @(Find-KeyPaths $jsonValue @("event_order", "eventOrder", "timestamp", "created_at", "createdAt", "time"))
}

$record = [ordered]@{
    captured_at = (Get-Date).ToUniversalTime().ToString("o")
    argv = @{
        event_name = $EventName
    }
    stdin = @{
        present = ($bytes.Length -gt 0)
        byte_length = $bytes.Length
        encoding = $decoded.Encoding
        is_json = $isJson
        parse_error = $parseError
        candidate_paths = $candidatePaths
        shape = if ($isJson) { Get-JsonShape $jsonValue } else { $null }
    }
}

$stamp = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssfffZ")
$path = Join-Path $OutDir "codex-lifecycle-hook-$stamp-$PID.json"
$record | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $path -Encoding UTF8
