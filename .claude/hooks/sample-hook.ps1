param(
    [Parameter(Mandatory = $true)]
    [string]$EventName
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$repoRoot = Split-Path -Parent $root
$exePath = Join-Path $repoRoot "taskbar-widget\target\debug\taskbar_widget_hook.exe"
$logDir = Join-Path $root "hook-logs"
$logPath = Join-Path $logDir "$EventName.jsonl"

if (-not (Test-Path -LiteralPath $exePath)) {
    throw "hook exe not found: $exePath"
}

New-Item -ItemType Directory -Force -Path $logDir | Out-Null

$raw = [Console]::In.ReadToEnd()
$sampleJson = $raw | & $exePath sample
$sample = $sampleJson | ConvertFrom-Json

$record = [ordered]@{
    at = (Get-Date).ToString("o")
    event = $EventName
    sample = $sample
}

($record | ConvertTo-Json -Depth 20 -Compress) | Add-Content -LiteralPath $logPath -Encoding utf8
