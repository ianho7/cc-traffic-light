param(
    [switch]$SkipBuild,
    [ValidateSet("debug", "release")]
    [string]$Configuration = "debug",
    [int]$TimeoutSeconds = 20,
    [string]$HostExePath
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$taskbarWidgetDir = Resolve-Path (Join-Path $scriptDir "..")
$repoRoot = Resolve-Path (Join-Path $taskbarWidgetDir "..")
$candidateTargetDirs = @(
    (Join-Path $repoRoot "target\$Configuration"),
    (Join-Path $taskbarWidgetDir "target\$Configuration")
)
$hostClassName = "TaskbarWidgetWindow"
$protocolVersion = "cc_traffic_light.settings.v1"
$pipeName = "\\.\pipe\cc-traffic-light-settings-v1"
$reportDir = Join-Path $taskbarWidgetDir "target\validate-tauri-settings-snapshot-alignment"
$reportPath = Join-Path $reportDir "report.json"

if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class SnapshotAlignmentUser32 {
    private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int GetClassNameW(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    public static IntPtr FindWindowRecursiveByClass(string className) {
        var found = IntPtr.Zero;
        EnumWindows((hWnd, _) => {
            if (HasClassName(hWnd, className)) {
                found = hWnd;
                return false;
            }

            EnumChildWindows(hWnd, (child, __) => {
                if (HasClassName(child, className)) {
                    found = child;
                    return false;
                }
                return true;
            }, IntPtr.Zero);

            return found == IntPtr.Zero;
        }, IntPtr.Zero);

        return found;
    }

    private static bool HasClassName(IntPtr hWnd, string expectedClassName) {
        var buffer = new StringBuilder(256);
        var length = GetClassNameW(hWnd, buffer, buffer.Capacity);
        return length > 0 && string.Equals(buffer.ToString(), expectedClassName, StringComparison.Ordinal);
    }
}

public static class SnapshotAlignmentPipeNative {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool WaitNamedPipeW(string name, uint timeoutMs);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool CallNamedPipeW(
        string lpNamedPipeName,
        byte[] lpInBuffer,
        uint nInBufferSize,
        byte[] lpOutBuffer,
        uint nOutBufferSize,
        out uint lpBytesRead,
        uint nTimeOut
    );
}
"@

function Resolve-ExecutableCandidate {
    param(
        [string[]]$TargetDirs,
        [string]$ExecutableName
    )

    foreach ($targetDir in $TargetDirs) {
        $candidate = Join-Path $targetDir $ExecutableName
        if (Test-Path $candidate) {
            return [System.IO.Path]::GetFullPath($candidate)
        }
    }

    return $null
}

function Wait-Until {
    param(
        [scriptblock]$Condition,
        [string]$Description,
        [int]$TimeoutSeconds = 20,
        [int]$PollMs = 200
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (& $Condition) {
            return
        }
        Start-Sleep -Milliseconds $PollMs
    }

    throw "Timed out waiting for: $Description"
}

function Find-HostWindow {
    [SnapshotAlignmentUser32]::FindWindowRecursiveByClass($hostClassName)
}

function Invoke-PipeCommand {
    param(
        [string]$RequestId,
        [Object]$Command
    )

    $envelope = [ordered]@{
        protocol_version = $protocolVersion
        request_id = $RequestId
        command = $Command
    }

    $payload = [System.Text.Encoding]::UTF8.GetBytes(($envelope | ConvertTo-Json -Compress -Depth 8))
    $null = [SnapshotAlignmentPipeNative]::WaitNamedPipeW($pipeName, 150)
    $responseBuffer = New-Object byte[] (64 * 1024)
    $bytesRead = 0
    $ok = [SnapshotAlignmentPipeNative]::CallNamedPipeW(
        $pipeName,
        $payload,
        [uint32]$payload.Length,
        $responseBuffer,
        [uint32]$responseBuffer.Length,
        [ref]$bytesRead,
        500
    )

    if (-not $ok) {
        throw "CallNamedPipeW failed for request $RequestId"
    }

    $json = [System.Text.Encoding]::UTF8.GetString($responseBuffer, 0, $bytesRead)
    return $json | ConvertFrom-Json
}

function Get-StateAgentSummary {
    param(
        [object]$State,
        [string]$SourceId
    )

    if (-not $State -or -not $State.agents) {
        return $null
    }

    $agentProperty = $State.agents.PSObject.Properties[$SourceId]
    if (-not $agentProperty -or -not $agentProperty.Value) {
        return $null
    }

    return $agentProperty.Value.summary
}

function Get-ExpectedSourceSnapshot {
    param(
        [object]$Summary,
        [string]$SourceId
    )

    if (-not $Summary) {
        return $null
    }

    $state = switch ($Summary.state) {
        "idle" { "idle" }
        "working" { "working" }
        "done" { "attention" }
        "waiting" { "blocking" }
        "error" { "blocking" }
        default { "undiscovered" }
    }

    $confidence = if ($Summary.has_stale) {
        "untrusted"
    } elseif ($Summary.state -eq "idle" -and [int]$Summary.active_task_count -eq 0) {
        "degraded"
    } else {
        "confirmed"
    }

    return [ordered]@{
        source_id = $SourceId
        state = $state
        confidence = $confidence
        method = "state_file"
        min_updated_at = [uint64]$Summary.updated_at
    }
}

function Test-SourceSnapshotMatch {
    param(
        [object]$Actual,
        [object]$Expected
    )

    if (-not $Actual -or -not $Expected) {
        return $false
    }

    return $Actual.state -eq $Expected.state `
        -and $Actual.confidence -eq $Expected.confidence `
        -and $Actual.method -eq $Expected.method `
        -and [uint64]$Actual.updated_at -ge [uint64]$Expected.min_updated_at
}

if (-not $SkipBuild) {
    Push-Location $repoRoot
    try {
        $cargoArgs = @("build", "--offline")
        if ($Configuration -eq "release") {
            $cargoArgs += "--release"
        }

        cargo @cargoArgs -p taskbar-widget
    }
    finally {
        Pop-Location
    }
}

$hostExe = if ($HostExePath) {
    [System.IO.Path]::GetFullPath($HostExePath)
} else {
    Resolve-ExecutableCandidate -TargetDirs $candidateTargetDirs -ExecutableName "taskbar-widget.exe"
}
if (-not $hostExe -or -not (Test-Path $hostExe)) {
    throw "Host executable not found in candidate target directories: $($candidateTargetDirs -join ', ')"
}

$statePath = Join-Path $env:APPDATA "CcTrafficLight\state.json"
$configPath = Join-Path $env:APPDATA "CcTrafficLight\config.json"
if (-not (Test-Path $statePath)) {
    throw "State file not found: $statePath"
}
if (-not (Test-Path $configPath)) {
    throw "Config file not found: $configPath"
}

$stateFile = Get-Content -Raw $statePath | ConvertFrom-Json
$configFile = Get-Content -Raw $configPath | ConvertFrom-Json
$expectedSources = [ordered]@{
    codex = Get-ExpectedSourceSnapshot -Summary (Get-StateAgentSummary -State $stateFile -SourceId "codex") -SourceId "codex"
    claude = Get-ExpectedSourceSnapshot -Summary (Get-StateAgentSummary -State $stateFile -SourceId "claude") -SourceId "claude"
}

$report = [ordered]@{
    timestamp = (Get-Date).ToString("s")
    configuration = $Configuration
    host_exe = $hostExe
    state_path = $statePath
    config_path = $configPath
    expected_sources = $expectedSources
    actual_sources = $null
    codex_snapshot_matches_state = $false
    claude_snapshot_matches_state = $false
    snapshot_matches_state_file = $false
    host_exit_code = $null
}

$startedHost = $null
$oldHostSetting = $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST

try {
    $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST = "tauri"
    $startedHost = Start-Process -FilePath $hostExe -WorkingDirectory (Split-Path -Parent $hostExe) -PassThru -WindowStyle Hidden

    Wait-Until -Description "host window to appear" -TimeoutSeconds $TimeoutSeconds -Condition {
        if ($startedHost.HasExited) {
            throw "Host exited before window appeared with code $($startedHost.ExitCode)"
        }

        (Find-HostWindow) -ne [IntPtr]::Zero
    }

    Wait-Until -Description "snapshot to reflect state file" -TimeoutSeconds $TimeoutSeconds -Condition {
        $snapshotResponse = Invoke-PipeCommand -RequestId "get-snapshot" -Command "get_snapshot"
        $actualSources = $snapshotResponse.response.get_snapshot.snapshot.sources
        $script:latestActualSources = $actualSources

        $codexEnabled = [bool]$configFile.monitoring.codex_enabled
        $claudeEnabled = [bool]$configFile.monitoring.claude_enabled
        $script:codexMatches = (-not $codexEnabled) -or (Test-SourceSnapshotMatch -Actual $actualSources.codex -Expected $expectedSources.codex)
        $script:claudeMatches = (-not $claudeEnabled) -or (Test-SourceSnapshotMatch -Actual $actualSources.claude -Expected $expectedSources.claude)

        $script:codexMatches -and $script:claudeMatches
    }

    $report.actual_sources = $script:latestActualSources
    $report.codex_snapshot_matches_state = $script:codexMatches
    $report.claude_snapshot_matches_state = $script:claudeMatches
    $report.snapshot_matches_state_file = $report.codex_snapshot_matches_state -and $report.claude_snapshot_matches_state

    Wait-Until -Description "host process to remain alive after snapshot check" -TimeoutSeconds 2 -Condition {
        -not $startedHost.HasExited
    }
}
finally {
    if ($startedHost) {
        if ($startedHost.HasExited) {
            $report.host_exit_code = $startedHost.ExitCode
        } else {
            Stop-Process -Id $startedHost.Id -Force -ErrorAction SilentlyContinue
            $startedHost.WaitForExit()
            $report.host_exit_code = $startedHost.ExitCode
        }
    }

    $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST = $oldHostSetting
}

$report | ConvertTo-Json -Depth 6 | Set-Content -Path $reportPath -Encoding UTF8
$report | ConvertTo-Json -Depth 6

if (-not $report.snapshot_matches_state_file) {
    exit 1
}
