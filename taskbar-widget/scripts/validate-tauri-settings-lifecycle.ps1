param(
    [switch]$SkipBuild,
    [ValidateSet("debug", "release")]
    [string]$Configuration = "debug",
    [int]$TimeoutSeconds = 20,
    [bool]$CloseToTray = $true
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$taskbarWidgetDir = Resolve-Path (Join-Path $scriptDir "..")
$repoRoot = Resolve-Path (Join-Path $taskbarWidgetDir "..")
$reportDir = Join-Path $taskbarWidgetDir "target\validate-tauri-settings-lifecycle"
$reportPath = Join-Path $reportDir "report.json"
$candidateTargetDirs = @(
    (Join-Path $repoRoot "target\$Configuration"),
    (Join-Path $taskbarWidgetDir "target\$Configuration")
)
$hostClassName = "TaskbarWidgetWindow"
$trayCmdOpenSettings = 1001
$wmCommand = 0x0111
$wmClose = 0x0010
$runtimeLogPath = Join-Path $reportDir "runtime.log"

if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

Add-Type -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class LifecycleUser32 {
    private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessageW(IntPtr hWnd, uint Msg, UIntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int GetClassNameW(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    public static IntPtr FindTopLevelWindowByProcessId(int processId) {
        var found = IntPtr.Zero;
        EnumWindows((hWnd, _) => {
            uint windowProcessId;
            GetWindowThreadProcessId(hWnd, out windowProcessId);
            if (windowProcessId == (uint)processId) {
                found = hWnd;
                return false;
            }

            return true;
        }, IntPtr.Zero);

        return found;
    }

    public static IntPtr FindVisibleTopLevelWindowByProcessId(int processId) {
        var found = IntPtr.Zero;
        EnumWindows((hWnd, _) => {
            uint windowProcessId;
            GetWindowThreadProcessId(hWnd, out windowProcessId);
            if (windowProcessId == (uint)processId && IsWindowVisible(hWnd)) {
                found = hWnd;
                return false;
            }

            return true;
        }, IntPtr.Zero);

        return found;
    }

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
        if (length <= 0) {
            return false;
        }
        return string.Equals(buffer.ToString(), expectedClassName, StringComparison.Ordinal);
    }
}
"@

function Resolve-ExecutableCandidate {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$TargetDirs,
        [Parameter(Mandatory = $true)]
        [string]$ExecutableName
    )

    foreach ($targetDir in $TargetDirs) {
        $directCandidate = Join-Path $targetDir $ExecutableName
        if (Test-Path $directCandidate) {
            return [System.IO.Path]::GetFullPath($directCandidate)
        }
    }

    foreach ($targetDir in $TargetDirs) {
        if (-not (Test-Path $targetDir)) {
            continue
        }

        $match = Get-ChildItem -Path $targetDir -Recurse -Filter $ExecutableName -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty FullName
        if ($match) {
            return [System.IO.Path]::GetFullPath($match)
        }
    }

    return $null
}

if (-not $SkipBuild) {
    Push-Location $repoRoot
    try {
        $cargoArgs = @("build", "--offline")
        if ($Configuration -eq "release") {
            $cargoArgs += "--release"
        }

        cargo @cargoArgs -p taskbar-settings-tauri
        cargo @cargoArgs -p taskbar-widget
    }
    finally {
        Pop-Location
    }
}

$hostExe = Resolve-ExecutableCandidate -TargetDirs $candidateTargetDirs -ExecutableName "taskbar-widget.exe"
$settingsExe = Resolve-ExecutableCandidate -TargetDirs $candidateTargetDirs -ExecutableName "taskbar-settings-tauri.exe"

if (-not $hostExe) {
    throw "Host executable not found in candidate target directories: $($candidateTargetDirs -join ', ')"
}
if (-not $settingsExe) {
    throw "Settings executable not found in candidate target directories: $($candidateTargetDirs -join ', ')"
}

$workspaceTargetDir = Split-Path -Parent $hostExe

function Get-MatchingProcesses {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProcessName,
        [Parameter(Mandatory = $true)]
        [string]$ExpectedPath
    )

    $normalizedPath = [System.IO.Path]::GetFullPath($ExpectedPath)
    @(
        Get-Process -Name $ProcessName -ErrorAction SilentlyContinue |
            Where-Object {
                try {
                    $_.Path -and ([System.IO.Path]::GetFullPath($_.Path) -eq $normalizedPath)
                }
                catch {
                    $false
                }
            }
    )
}

function Wait-Until {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$Condition,
        [Parameter(Mandatory = $true)]
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
    return [LifecycleUser32]::FindWindowRecursiveByClass($hostClassName)
}

function Find-SettingsWindow {
    $settingsProcesses = Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe
    if ($settingsProcesses.Count -ne 1) {
        return [IntPtr]::Zero
    }

    # The host's native fallback uses the same title, so title-only lookup is
    # unsafe. Always resolve the window through the settings process PID.
    return [LifecycleUser32]::FindVisibleTopLevelWindowByProcessId($settingsProcesses[0].Id)
}

function Send-HostOpenSettings {
    $hwnd = Find-HostWindow
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "Host window not found for class $hostClassName"
    }
    $ok = [LifecycleUser32]::PostMessageW(
        $hwnd,
        [uint32]$wmCommand,
        [System.UIntPtr]::new([uint32]$trayCmdOpenSettings),
        [IntPtr]::Zero
    )
    if (-not $ok) {
        throw "PostMessageW(WM_COMMAND, TRAY_CMD_OPEN_SETTINGS) failed"
    }
}

function Send-HostClose {
    $hwnd = Find-HostWindow
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "Host window not found for class $hostClassName"
    }
    $ok = [LifecycleUser32]::PostMessageW($hwnd, [uint32]$wmClose, [System.UIntPtr]::Zero, [IntPtr]::Zero)
    if (-not $ok) {
        throw "PostMessageW(WM_CLOSE) failed for host"
    }
}

function Send-SettingsClose {
    $hwnd = Find-SettingsWindow
    if ($hwnd -eq [IntPtr]::Zero) {
        throw "Settings window not found for the managed settings process"
    }
    $ok = [LifecycleUser32]::PostMessageW($hwnd, [uint32]$wmClose, [System.UIntPtr]::Zero, [IntPtr]::Zero)
    if (-not $ok) {
        throw "PostMessageW(WM_CLOSE) failed"
    }
}

function Test-SettingsWindowVisible {
    $hwnd = Find-SettingsWindow
    return $hwnd -ne [IntPtr]::Zero -and [LifecycleUser32]::IsWindowVisible($hwnd)
}

$hostProcessName = [System.IO.Path]::GetFileNameWithoutExtension($hostExe)
$settingsProcessName = [System.IO.Path]::GetFileNameWithoutExtension($settingsExe)
$startedHost = $null
$report = [ordered]@{
    timestamp = (Get-Date).ToString("s")
    configuration = $Configuration
    host_exe = $hostExe
    settings_exe = $settingsExe
    runtime_log = $runtimeLogPath
    host_started = $false
    host_exit_code = $null
    open_spawned_once = $false
    reopen_reused_same_process = $false
    close_exited_process = $false
    close_kept_process_running = $false
    reopen_after_close_reused_process = $false
    reopen_after_close_spawned = $false
    reopen_after_forced_kill_spawned = $false
    host_shutdown_reaped_settings = $false
    first_settings_pid = $null
    second_settings_pid = $null
    third_settings_pid = $null
    notes = @()
}

$oldHostSetting = $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST
$oldSettingsExe = $env:CC_TRAFFIC_LIGHT_TAURI_SETTINGS_EXE
$oldRuntimeLog = $env:TASKBAR_MVP_RUNTIME_LOG_FILE
$oldAppData = $env:APPDATA
$isolatedAppData = Join-Path $reportDir ("appdata-" + [Guid]::NewGuid().ToString("N"))
$configPath = Join-Path $isolatedAppData "CcTrafficLight\config.json"

try {
    if ((Get-MatchingProcesses -ProcessName $hostProcessName -ExpectedPath $hostExe).Count -gt 0) {
        throw "Existing host process already running from $hostExe; aborting validation to avoid interfering with user session."
    }
    if ((Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -gt 0) {
        throw "Existing settings process already running from $settingsExe; aborting validation to avoid interfering with user session."
    }

    $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST = "tauri"
    $env:CC_TRAFFIC_LIGHT_TAURI_SETTINGS_EXE = $settingsExe
    $env:TASKBAR_MVP_RUNTIME_LOG_FILE = $runtimeLogPath
    $env:APPDATA = $isolatedAppData
    New-Item -ItemType Directory -Path (Split-Path -Parent $configPath) -Force | Out-Null
    @{ schema_version = 7; general = @{ autostart_enabled = $false; close_to_tray = $CloseToTray } } |
        ConvertTo-Json -Depth 4 | Set-Content -Path $configPath -Encoding UTF8
    $report.close_to_tray = $CloseToTray
    $report.config_path = $configPath
    if (Test-Path $runtimeLogPath) {
        Remove-Item -LiteralPath $runtimeLogPath -Force
    }

    $startedHost = Start-Process -FilePath $hostExe -WorkingDirectory $workspaceTargetDir -PassThru -WindowStyle Hidden
    $report.host_started = $true
    $report.notes += "Started host pid=$($startedHost.Id)"

    Wait-Until -Description "host window to appear" -TimeoutSeconds $TimeoutSeconds -Condition {
        if ($startedHost.HasExited) {
            $report.host_exit_code = $startedHost.ExitCode
            throw "Host process exited before window appeared with code $($startedHost.ExitCode)"
        }
        (Find-HostWindow) -ne [IntPtr]::Zero
    }

    Send-HostOpenSettings
    Wait-Until -Description "first settings process" -TimeoutSeconds $TimeoutSeconds -Condition {
        (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 1
    }
    $firstProcesses = Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe
    $report.first_settings_pid = $firstProcesses[0].Id
    $report.open_spawned_once = $true
    Wait-Until -Description "first settings window" -TimeoutSeconds $TimeoutSeconds -Condition {
        (Find-SettingsWindow) -ne [IntPtr]::Zero
    }
    # The frontend invokes bootstrap_window after the native Tauri window exists;
    # allow that initial settings read to synchronize close_to_tray before testing
    # the close request.
    Start-Sleep -Milliseconds 800

    Send-HostOpenSettings
    Start-Sleep -Milliseconds 800
    $secondOpenProcesses = Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe
    if ($secondOpenProcesses.Count -eq 1 -and $secondOpenProcesses[0].Id -eq $report.first_settings_pid) {
        $report.reopen_reused_same_process = $true
    }
    else {
        throw "Repeated open did not reuse a single settings process."
    }

    Send-SettingsClose
    if ($CloseToTray) {
        Wait-Until -Description "settings process to remain after close-to-tray" -TimeoutSeconds $TimeoutSeconds -Condition {
            (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 1
        }
        $report.close_kept_process_running = $true

        Send-HostOpenSettings
        Wait-Until -Description "hidden settings window to be restored" -TimeoutSeconds $TimeoutSeconds -Condition {
            Test-SettingsWindowVisible
        }
        $reopenedProcesses = Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe
        if ($reopenedProcesses[0].Id -ne $report.first_settings_pid) {
            throw "Close-to-tray did not reuse the existing settings process."
        }
        $report.reopen_after_close_reused_process = $true
        $settingsPidToKill = $report.first_settings_pid
    }
    else {
        Wait-Until -Description "settings process to exit after close" -TimeoutSeconds $TimeoutSeconds -Condition {
            (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 0
        }
        $report.close_exited_process = $true

        Send-HostOpenSettings
        Wait-Until -Description "settings process to respawn after close" -TimeoutSeconds $TimeoutSeconds -Condition {
            (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 1
        }
        $secondProcesses = Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe
        $report.second_settings_pid = $secondProcesses[0].Id
        $report.reopen_after_close_spawned = $true
        $settingsPidToKill = $report.second_settings_pid
    }

    Stop-Process -Id $settingsPidToKill -Force
    Wait-Until -Description "forced-killed settings process to exit" -TimeoutSeconds $TimeoutSeconds -Condition {
        (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 0
    }

    Send-HostOpenSettings
    Wait-Until -Description "settings process to respawn after forced kill" -TimeoutSeconds $TimeoutSeconds -Condition {
        (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 1
    }
    $thirdProcesses = Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe
    $report.third_settings_pid = $thirdProcesses[0].Id
    $report.reopen_after_forced_kill_spawned = $true

    Send-HostClose
    Wait-Until -Description "host process to exit after WM_CLOSE" -TimeoutSeconds $TimeoutSeconds -Condition {
        $startedHost.HasExited
    }
    $report.host_exit_code = $startedHost.ExitCode
    $startedHost = $null
    Wait-Until -Description "settings process to exit after host shutdown" -TimeoutSeconds $TimeoutSeconds -Condition {
        (Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe).Count -eq 0
    }
    $report.host_shutdown_reaped_settings = $true
}
finally {
    if ($startedHost -and -not $startedHost.HasExited) {
        Stop-Process -Id $startedHost.Id -Force -ErrorAction SilentlyContinue
    }

    Get-MatchingProcesses -ProcessName $settingsProcessName -ExpectedPath $settingsExe |
        ForEach-Object {
            Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        }

    $env:CC_TRAFFIC_LIGHT_SETTINGS_HOST = $oldHostSetting
    $env:CC_TRAFFIC_LIGHT_TAURI_SETTINGS_EXE = $oldSettingsExe
    $env:TASKBAR_MVP_RUNTIME_LOG_FILE = $oldRuntimeLog
    $env:APPDATA = $oldAppData

    $report | ConvertTo-Json -Depth 4 | Set-Content -Path $reportPath -Encoding UTF8
}

$report
