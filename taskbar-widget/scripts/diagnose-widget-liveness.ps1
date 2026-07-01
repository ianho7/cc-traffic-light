param(
    [ValidateSet("baseline", "fixture_replay", "real_live")]
    [string]$LoopType = "baseline",
    [switch]$SkipBuild,
    [string]$OutDir = "",
    [string]$StateDir = "",
    [int[]]$BaselineSampleOffsetsMs = @(0, 1000, 2000, 3000, 5000),
    [int[]]$RealSampleOffsetsMs = @(0, 500, 1000, 2000, 3000),
    [int]$FixtureStepDelayMs = 1200
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $runId = "{0}-{1}" -f $LoopType, (Get-Date -Format "yyyyMMdd-HHmmss")
    $OutDir = Join-Path $projectRoot "target\diagnose-widget-liveness\$runId"
}
$null = New-Item -ItemType Directory -Force -Path $OutDir

if (-not $SkipBuild) {
    Push-Location $projectRoot
    try {
        cargo build | Out-Null
    } finally {
        Pop-Location
    }
}

$widgetExe = Join-Path $projectRoot "target\debug\taskbar-widget.exe"
$hookExe = Join-Path $projectRoot "target\debug\taskbar_widget_hook.exe"
if (-not (Test-Path $widgetExe)) {
    throw "Executable not found: $widgetExe"
}
if (-not (Test-Path $hookExe)) {
    throw "Hook executable not found: $hookExe"
}

Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class DiagnoseWidgetNative {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);

    [DllImport("user32.dll")]
    public static extern bool IsWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern IntPtr SendMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
}
'@

function Convert-RectString {
    param([string]$RectString)

    if ([string]::IsNullOrWhiteSpace($RectString)) {
        return $null
    }

    $parts = $RectString.Split(",")
    if ($parts.Count -ne 4) {
        return $null
    }

    [pscustomobject]@{
        Left = [int]$parts[0]
        Top = [int]$parts[1]
        Right = [int]$parts[2]
        Bottom = [int]$parts[3]
    }
}

function Convert-HexHandle {
    param([string]$Hex)

    if ([string]::IsNullOrWhiteSpace($Hex) -or $Hex -eq "0x0") {
        return [IntPtr]::Zero
    }

    return [IntPtr]([Convert]::ToInt64($Hex.Replace("0x", ""), 16))
}

function Get-ScreenCapture {
    param(
        [object]$Rect,
        [string]$Path
    )

    if (-not $Rect) {
        throw "Screen capture rect missing"
    }

    $width = [Math]::Max(1, $Rect.Right - $Rect.Left)
    $height = [Math]::Max(1, $Rect.Bottom - $Rect.Top)
    $bmp = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bmp)
    $graphics.CopyFromScreen($Rect.Left, $Rect.Top, 0, 0, $bmp.Size)
    $graphics.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

function Measure-CaptureStats {
    param([string]$Path)

    if (-not (Test-Path $Path)) {
        return [pscustomobject]@{
            BrightPixels = 0
            MeanBrightness = 0.0
            HasVisibleText = $false
        }
    }

    $bmp = [System.Drawing.Bitmap]::FromFile($Path)
    try {
        $bright = 0
        $sum = 0.0
        for ($x = 0; $x -lt $bmp.Width; $x++) {
            for ($y = 0; $y -lt $bmp.Height; $y++) {
                $pixel = $bmp.GetPixel($x, $y)
                $sum += ($pixel.R + $pixel.G + $pixel.B) / 3.0
                if ($pixel.R -gt 220 -and $pixel.G -gt 220 -and $pixel.B -gt 220) {
                    $bright++
                }
            }
        }

        $mean = $sum / [Math]::Max(1, ($bmp.Width * $bmp.Height))
        [pscustomobject]@{
            BrightPixels = $bright
            MeanBrightness = [Math]::Round($mean, 2)
            HasVisibleText = ($bright -gt 20)
        }
    } finally {
        $bmp.Dispose()
    }
}

function Get-LastRuntimeLogLine {
    param([string]$RuntimeLogPath)

    if (-not (Test-Path $RuntimeLogPath)) {
        return ""
    }

    @(Get-Content -LiteralPath $RuntimeLogPath -Tail 1) -join ""
}

function Invoke-HookCli {
    param(
        [string]$Command,
        [string[]]$Arguments,
        [string]$StateHome
    )

    $previousStateHome = $env:TASKBAR_WIDGET_STATE_HOME
    try {
        if ([string]::IsNullOrWhiteSpace($StateHome)) {
            Remove-Item Env:TASKBAR_WIDGET_STATE_HOME -ErrorAction SilentlyContinue
        } else {
            $env:TASKBAR_WIDGET_STATE_HOME = $StateHome
        }

        & $hookExe $Command @Arguments
    } finally {
        if ($null -eq $previousStateHome) {
            Remove-Item Env:TASKBAR_WIDGET_STATE_HOME -ErrorAction SilentlyContinue
        } else {
            $env:TASKBAR_WIDGET_STATE_HOME = $previousStateHome
        }
    }
}

function Get-StateSnapshot {
    param([string]$StateHome)

    $json = Invoke-HookCli -Command "list" -Arguments @() -StateHome $StateHome
    $state = $json | ConvertFrom-Json
    [pscustomobject]@{
        Raw = $state
        Summary = $state.global_summary
        ActiveTaskCount = [int]$state.global_summary.active_task_count
        State = [string]$state.global_summary.state
        UpdatedAt = [uint64]$state.global_summary.updated_at
    }
}

function Reset-IsolatedStateHome {
    param([string]$StateHome)

    if ([string]::IsNullOrWhiteSpace($StateHome)) {
        throw "Reset-IsolatedStateHome requires a concrete path"
    }

    if (Test-Path $StateHome) {
        Remove-Item -LiteralPath $StateHome -Recurse -Force
    }
    $null = New-Item -ItemType Directory -Force -Path $StateHome
}

function Set-WaitingFixtureState {
    param(
        [string]$StateHome,
        [int]$Count
    )

    for ($index = 1; $index -le $Count; $index++) {
        $taskKey = "codex_fixture_{0:d2}" -f $index
        $null = Invoke-HookCli -Command "set" -Arguments @($taskKey, "waiting") -StateHome $StateHome
    }
}

function Test-PidAlive {
    param([int]$ProcessId)

    @(Get-Process -Id $ProcessId -ErrorAction SilentlyContinue).Count -gt 0
}

function Test-HwndValid {
    param([string]$HexHandle)

    $hwnd = Convert-HexHandle $HexHandle
    if ($hwnd -eq [IntPtr]::Zero) {
        return $false
    }

    [DiagnoseWidgetNative]::IsWindow($hwnd)
}

function Start-WidgetInstance {
    param(
        [string]$RunDir,
        [string]$StateHome
    )

    $diagPath = Join-Path $RunDir "widget-diag.json"
    $runtimeLogPath = Join-Path $RunDir "runtime.log"
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $widgetExe
    $psi.WorkingDirectory = Split-Path -Parent $widgetExe
    $psi.UseShellExecute = $false
    $psi.EnvironmentVariables["TASKBAR_MVP_DIAG_FILE"] = $diagPath
    $psi.EnvironmentVariables["TASKBAR_MVP_RUNTIME_LOG_FILE"] = $runtimeLogPath
    if (-not [string]::IsNullOrWhiteSpace($StateHome)) {
        $psi.EnvironmentVariables["TASKBAR_WIDGET_STATE_HOME"] = $StateHome
    }
    $proc = [System.Diagnostics.Process]::Start($psi)

    $deadline = (Get-Date).AddSeconds(5)
    while ((Get-Date) -lt $deadline -and -not (Test-Path $diagPath)) {
        Start-Sleep -Milliseconds 100
    }

    if (-not (Test-Path $diagPath)) {
        throw "Widget diagnostics file not written: $diagPath"
    }

    $diag = Get-Content -Raw $diagPath | ConvertFrom-Json
    [pscustomobject]@{
        Pid = $proc.Id
        DiagPath = $diagPath
        RuntimeLogPath = $runtimeLogPath
        Diag = $diag
    }
}

function Stop-WidgetInstance {
    param([pscustomobject]$Instance)

    $hwnd = Convert-HexHandle $Instance.Diag.window.hwnd
    if ($hwnd -ne [IntPtr]::Zero -and [DiagnoseWidgetNative]::IsWindow($hwnd)) {
        $null = [DiagnoseWidgetNative]::SendMessage($hwnd, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
        Start-Sleep -Milliseconds 400
    }

    if (Test-PidAlive -ProcessId $Instance.Pid) {
        Stop-Process -Id $Instance.Pid -Force -ErrorAction SilentlyContinue
    }
}

function New-Checkpoint {
    param(
        [pscustomobject]$Instance,
        [string]$RunDir,
        [string]$StateHome,
        [string]$Name,
        [int]$OffsetMs
    )

    $summary = Get-StateSnapshot -StateHome $StateHome
    $capturePath = Join-Path $RunDir "$Name.png"
    $moduleRect = Convert-RectString $Instance.Diag.layout.module_rect
    $captureError = ""
    try {
        Get-ScreenCapture -Rect $moduleRect -Path $capturePath
    } catch {
        $captureError = $_.Exception.Message
    }
    $captureStats = Measure-CaptureStats -Path $capturePath
    $runtimeTail = Get-LastRuntimeLogLine -RuntimeLogPath $Instance.RuntimeLogPath

    [pscustomobject]@{
        Name = $Name
        OffsetMs = $OffsetMs
        PidAlive = (Test-PidAlive -ProcessId $Instance.Pid)
        HwndValid = (Test-HwndValid -HexHandle $Instance.Diag.window.hwnd)
        SummaryState = $summary.State
        ActiveTaskCount = $summary.ActiveTaskCount
        SummaryUpdatedAt = $summary.UpdatedAt
        CapturePath = $capturePath
        CaptureError = $captureError
        BrightPixels = $captureStats.BrightPixels
        MeanBrightness = $captureStats.MeanBrightness
        CaptureHasVisibleText = $captureStats.HasVisibleText
        LastRuntimeLogLine = $runtimeTail
    }
}

function Test-CheckpointHealthy {
    param([pscustomobject]$Checkpoint)

    $Checkpoint.PidAlive -and $Checkpoint.HwndValid -and $Checkpoint.CaptureHasVisibleText -and
        -not ($Checkpoint.LastRuntimeLogLine -match "WM_DESTROY|WM_NCDESTROY|message loop exited cleanly")
}

function Invoke-BaselineLoop {
    $stateHome = if ([string]::IsNullOrWhiteSpace($StateDir)) {
        Join-Path $OutDir "state"
    } else {
        $StateDir
    }

    Reset-IsolatedStateHome -StateHome $stateHome
    $instance = Start-WidgetInstance -RunDir $OutDir -StateHome $stateHome
    try {
        $checkpoints = New-Object System.Collections.Generic.List[object]
        $lastOffset = 0
        foreach ($offset in $BaselineSampleOffsetsMs) {
            $sleepMs = [Math]::Max(0, $offset - $lastOffset)
            if ($sleepMs -gt 0) {
                Start-Sleep -Milliseconds $sleepMs
            }
            $checkpoints.Add((New-Checkpoint -Instance $instance -RunDir $OutDir -StateHome $stateHome -Name ("checkpoint-{0:d4}ms" -f $offset) -OffsetMs $offset)) | Out-Null
            $lastOffset = $offset
        }

        $pass = $true
        $reason = "baseline stable for all checkpoints"
        foreach ($checkpoint in $checkpoints) {
            if (-not (Test-CheckpointHealthy -Checkpoint $checkpoint)) {
                $pass = $false
                $reason = "checkpoint {0} lost pid/hwnd/text or logged exit" -f $checkpoint.Name
                break
            }
        }

        [pscustomobject]@{
            loop_type = "baseline"
            state_home = $stateHome
            pid = $instance.Pid
            hwnd = $instance.Diag.window.hwnd
            module_rect = $instance.Diag.layout.module_rect
            runtime_log = $instance.RuntimeLogPath
            summary_before = @{
                state = $checkpoints[0].SummaryState
                active_task_count = $checkpoints[0].ActiveTaskCount
            }
            summary_after = @{
                state = $checkpoints[$checkpoints.Count - 1].SummaryState
                active_task_count = $checkpoints[$checkpoints.Count - 1].ActiveTaskCount
            }
            checkpoints = $checkpoints
            pid_alive = $checkpoints[$checkpoints.Count - 1].PidAlive
            hwnd_valid = $checkpoints[$checkpoints.Count - 1].HwndValid
            runtime_log_tail = $checkpoints[$checkpoints.Count - 1].LastRuntimeLogLine
            capture_paths = $checkpoints | ForEach-Object { $_.CapturePath }
            result = @{
                pass = $pass
                reason = $reason
            }
        }
    } finally {
        Stop-WidgetInstance -Instance $instance
    }
}

function Invoke-FixtureReplayLoop {
    $stateHome = if ([string]::IsNullOrWhiteSpace($StateDir)) {
        Join-Path $OutDir "state"
    } else {
        $StateDir
    }

    Reset-IsolatedStateHome -StateHome $stateHome
    Set-WaitingFixtureState -StateHome $stateHome -Count 2
    $instance = Start-WidgetInstance -RunDir $OutDir -StateHome $stateHome
    try {
        $checkpoints = New-Object System.Collections.Generic.List[object]
        $checkpoints.Add((New-Checkpoint -Instance $instance -RunDir $OutDir -StateHome $stateHome -Name "fixture-waiting-2" -OffsetMs 0)) | Out-Null

        Start-Sleep -Milliseconds $FixtureStepDelayMs
        Set-WaitingFixtureState -StateHome $stateHome -Count 3
        Start-Sleep -Milliseconds $FixtureStepDelayMs
        $checkpoints.Add((New-Checkpoint -Instance $instance -RunDir $OutDir -StateHome $stateHome -Name "fixture-waiting-3" -OffsetMs ($FixtureStepDelayMs * 2))) | Out-Null

        Set-WaitingFixtureState -StateHome $stateHome -Count 4
        Start-Sleep -Milliseconds $FixtureStepDelayMs
        $checkpoints.Add((New-Checkpoint -Instance $instance -RunDir $OutDir -StateHome $stateHome -Name "fixture-waiting-4" -OffsetMs ($FixtureStepDelayMs * 3))) | Out-Null

        $expectedCounts = @(2, 3, 4)
        $pass = $true
        $reason = "fixture replay reached WAITING 2 -> 3 -> 4"
        for ($index = 0; $index -lt $checkpoints.Count; $index++) {
            if ($checkpoints[$index].ActiveTaskCount -ne $expectedCounts[$index]) {
                $pass = $false
                $reason = "checkpoint {0} expected active_task_count={1} got {2}" -f $checkpoints[$index].Name, $expectedCounts[$index], $checkpoints[$index].ActiveTaskCount
                break
            }
            if (-not (Test-CheckpointHealthy -Checkpoint $checkpoints[$index])) {
                $pass = $false
                $reason = "checkpoint {0} lost pid/hwnd/text or logged exit" -f $checkpoints[$index].Name
                break
            }
        }

        [pscustomobject]@{
            loop_type = "fixture_replay"
            state_home = $stateHome
            pid = $instance.Pid
            hwnd = $instance.Diag.window.hwnd
            module_rect = $instance.Diag.layout.module_rect
            runtime_log = $instance.RuntimeLogPath
            summary_before = @{
                state = $checkpoints[0].SummaryState
                active_task_count = $checkpoints[0].ActiveTaskCount
            }
            summary_after = @{
                state = $checkpoints[$checkpoints.Count - 1].SummaryState
                active_task_count = $checkpoints[$checkpoints.Count - 1].ActiveTaskCount
            }
            checkpoints = $checkpoints
            pid_alive = $checkpoints[$checkpoints.Count - 1].PidAlive
            hwnd_valid = $checkpoints[$checkpoints.Count - 1].HwndValid
            runtime_log_tail = $checkpoints[$checkpoints.Count - 1].LastRuntimeLogLine
            capture_paths = $checkpoints | ForEach-Object { $_.CapturePath }
            result = @{
                pass = $pass
                reason = $reason
            }
        }
    } finally {
        Stop-WidgetInstance -Instance $instance
    }
}

function Invoke-RealLiveLoop {
    $stateHome = $StateDir
    $instance = Start-WidgetInstance -RunDir $OutDir -StateHome $stateHome
    try {
        $checkpoints = New-Object System.Collections.Generic.List[object]
        $lastOffset = 0
        foreach ($offset in $RealSampleOffsetsMs) {
            $sleepMs = [Math]::Max(0, $offset - $lastOffset)
            if ($sleepMs -gt 0) {
                Start-Sleep -Milliseconds $sleepMs
            }
            $checkpoints.Add((New-Checkpoint -Instance $instance -RunDir $OutDir -StateHome $stateHome -Name ("checkpoint-{0:d4}ms" -f $offset) -OffsetMs $offset)) | Out-Null
            $lastOffset = $offset
        }

        $initialCount = $checkpoints[0].ActiveTaskCount
        $changedCheckpoint = $checkpoints | Where-Object { $_.ActiveTaskCount -gt $initialCount } | Select-Object -First 1
        $pass = $false
        $reason = "no real-state change observed within sampling window"
        if ($changedCheckpoint) {
            if (Test-CheckpointHealthy -Checkpoint $changedCheckpoint) {
                $pass = $true
                $reason = "real-state active_task_count changed from $initialCount to $($changedCheckpoint.ActiveTaskCount) and remained visible"
            } else {
                $reason = "state changed but checkpoint $($changedCheckpoint.Name) lost pid/hwnd/text or logged exit"
            }
        }

        [pscustomobject]@{
            loop_type = "real_live"
            state_home = if ([string]::IsNullOrWhiteSpace($stateHome)) { "<default_appdata>" } else { $stateHome }
            pid = $instance.Pid
            hwnd = $instance.Diag.window.hwnd
            module_rect = $instance.Diag.layout.module_rect
            runtime_log = $instance.RuntimeLogPath
            summary_before = @{
                state = $checkpoints[0].SummaryState
                active_task_count = $checkpoints[0].ActiveTaskCount
            }
            summary_after = @{
                state = $checkpoints[$checkpoints.Count - 1].SummaryState
                active_task_count = $checkpoints[$checkpoints.Count - 1].ActiveTaskCount
            }
            checkpoints = $checkpoints
            pid_alive = $checkpoints[$checkpoints.Count - 1].PidAlive
            hwnd_valid = $checkpoints[$checkpoints.Count - 1].HwndValid
            runtime_log_tail = $checkpoints[$checkpoints.Count - 1].LastRuntimeLogLine
            capture_paths = $checkpoints | ForEach-Object { $_.CapturePath }
            result = @{
                pass = $pass
                reason = $reason
            }
        }
    } finally {
        Stop-WidgetInstance -Instance $instance
    }
}

$report = switch ($LoopType) {
    "baseline" { Invoke-BaselineLoop }
    "fixture_replay" { Invoke-FixtureReplayLoop }
    "real_live" { Invoke-RealLiveLoop }
}

$reportPath = Join-Path $OutDir "report.json"
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath

[pscustomobject]@{
    LoopType = $report.loop_type
    Pass = $report.result.pass
    Reason = $report.result.reason
    Pid = $report.pid
    Hwnd = $report.hwnd
    Before = "{0} {1}" -f $report.summary_before.state, $report.summary_before.active_task_count
    After = "{0} {1}" -f $report.summary_after.state, $report.summary_after.active_task_count
} | Format-Table -AutoSize

Write-Host ""
Write-Host "Report JSON: $reportPath"

if (-not $report.result.pass) {
    exit 1
}
