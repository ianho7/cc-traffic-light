param(
    [string]$HookName = "unknown"
)

$ErrorActionPreference = "Stop"

$dumpDir = "$env:TEMP\cc-traffic-light-claude-hooks"
if (-not (Test-Path -LiteralPath $dumpDir)) {
    New-Item -ItemType Directory -Path $dumpDir | Out-Null
}

$timestamp = [DateTime]::UtcNow.ToString("yyyyMMdd-HHmmss-fffffff")
$dumpFile = Join-Path $dumpDir "claude-hook-$HookName-$timestamp.json"

try {
    $stdin = [Console]::OpenStandardInput()
    $reader = [System.IO.StreamReader]::new($stdin, [System.Text.Encoding]::UTF8)
    $inputText = $reader.ReadToEnd()
} catch {
    $inputText = "(stdin read failed: $($_.Exception.Message))"
}

$record = @{
    hook_name = $HookName
    received_at = [DateTime]::UtcNow.ToString("o")
    stdin_present = $inputText.Length -gt 0
    stdin_is_json = $false
    stdin_parse_error = $null
    stdin_candidate_paths = @{
        session = @()
        turn = @()
        hook_event = @()
        event_order = @()
    }
    stdin_raw_character_count = $inputText.Length
    stdin_sample = "redacted; shape-only capture"
}

# Try parsing as JSON to extract shape
if ($inputText.Trim().Length -gt 0) {
    try {
        $json = $inputText | ConvertFrom-Json
        $record.stdin_is_json = $true

        # Recursively find candidate field paths
        function Find-KeyPaths {
            param($Obj, [string]$Prefix, [string[]]$Keys)
            $results = @()
            if ($null -eq $Obj) { return $results }
            foreach ($prop in $Obj.PSObject.Properties) {
                $path = if ($Prefix) { "$Prefix.$($prop.Name)" } else { $prop.Name }
                if ($Keys -contains $prop.Name) {
                    $results += $path
                }
                if ($null -ne $prop.Value -and ($prop.Value -is [PSCustomObject])) {
                    $results += Find-KeyPaths -Obj $prop.Value -Prefix $path -Keys $Keys
                }
            }
            $results
        }

        $record.stdin_candidate_paths.session = @(Find-KeyPaths -Obj $json -Prefix "" -Keys @("session_id", "sessionId", "sessionID"))
        $record.stdin_candidate_paths.turn = @(Find-KeyPaths -Obj $json -Prefix "" -Keys @("turn_id", "turnId", "turnID"))
        $record.stdin_candidate_paths.hook_event = @(Find-KeyPaths -Obj $json -Prefix "" -Keys @("hook_event_name", "hookName", "eventName", "hook_name"))
        $record.stdin_candidate_paths.event_order = @(Find-KeyPaths -Obj $json -Prefix "" -Keys @("event_order", "eventOrder", "timestamp", "created_at", "createdAt", "time"))

        # Also record top-level keys for shape understanding
        $record.stdin_top_level_keys = @($json.PSObject.Properties.Name)
    } catch {
        $record.stdin_is_json = $false
        $record.stdin_parse_error = $_.Exception.Message
    }
}

$record | ConvertTo-Json -Depth 20 | Out-File -LiteralPath $dumpFile -Encoding UTF8

# Notify via log (will appear in Codex output)
Write-Host "[cc-traffic-light] dumped claude $HookName hook to $dumpFile"
