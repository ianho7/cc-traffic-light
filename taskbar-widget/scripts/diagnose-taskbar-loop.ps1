param(
    [string[]]$Parents = @("shell", "rebar", "task_switch", "composition_bridge"),
    [string[]]$CoordModes = @("rect_delta", "screen_to_client"),
    [string[]]$Anchors = @("tray_notify", "task_switch", "start"),
    [switch]$SkipBuild,
    [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $OutDir = Join-Path $projectRoot "target\diagnose-taskbar-loop"
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

$exe = Join-Path $projectRoot "target\debug\taskbar-widget.exe"
if (-not (Test-Path $exe)) {
    throw "Executable not found: $exe"
}

Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class DiagnoseTaskbarNative {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindowEx(IntPtr parent, IntPtr childAfter, string className, string windowName);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")]
    public static extern bool PrintWindow(IntPtr hwnd, IntPtr hDC, uint nFlags);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr SendMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumChildWindows(IntPtr hWndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    public static string ClassName(IntPtr hwnd) {
        var sb = new StringBuilder(256);
        GetClassName(hwnd, sb, sb.Capacity);
        return sb.ToString();
    }
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

function Get-TaskbarCapture {
    param(
        [IntPtr]$Window,
        [string]$Path
    )

    if ($Window -eq [IntPtr]::Zero) {
        throw "Capture window not found"
    }

    $rect = New-Object DiagnoseTaskbarNative+RECT
    $null = [DiagnoseTaskbarNative]::GetWindowRect($Window, [ref]$rect)

    $width = [Math]::Max(1, $rect.Right - $rect.Left)
    $height = [Math]::Max(1, $rect.Bottom - $rect.Top)
    $bmp = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bmp)
    $hdc = $graphics.GetHdc()
    $ok = [DiagnoseTaskbarNative]::PrintWindow($Window, $hdc, 0)
    $graphics.ReleaseHdc($hdc)
    $graphics.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()

    [pscustomobject]@{
        Window = $Window
        Rect = [pscustomobject]@{
            Left = $rect.Left
            Top = $rect.Top
            Right = $rect.Right
            Bottom = $rect.Bottom
        }
        Printed = $ok
        Path = $Path
    }
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

    [pscustomobject]@{
        Rect = $Rect
        Path = $Path
    }
}

function Get-RustTaskbarChildren {
    $shell = [DiagnoseTaskbarNative]::FindWindow("Shell_TrayWnd", $null)
    if ($shell -eq [IntPtr]::Zero) {
        return @()
    }

    $children = New-Object System.Collections.Generic.List[object]
    $callback = [DiagnoseTaskbarNative+EnumWindowsProc]{
        param($hwnd, $lParam)
        if ([DiagnoseTaskbarNative]::ClassName($hwnd) -eq "TaskbarWidgetWindow") {
            $children.Add($hwnd) | Out-Null
        }
        return $true
    }
    $null = [DiagnoseTaskbarNative]::EnumChildWindows($shell, $callback, [IntPtr]::Zero)
    return $children.ToArray()
}

function Clear-StaleTaskbarChildren {
    Get-Process taskbar-widget -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    $children = Get-RustTaskbarChildren
    foreach ($child in $children) {
        $null = [DiagnoseTaskbarNative]::SendMessage($child, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
    }
    Start-Sleep -Milliseconds 400
}

function Get-CandidateParentHandle {
    param([string]$Parent)

    $shell = [DiagnoseTaskbarNative]::FindWindow("Shell_TrayWnd", $null)
    if ($shell -eq [IntPtr]::Zero) {
        return [IntPtr]::Zero
    }

    switch ($Parent) {
        "shell" { return $shell }
        "rebar" {
            $rebar = [DiagnoseTaskbarNative]::FindWindowEx($shell, [IntPtr]::Zero, "ReBarWindow32", $null)
            if ($rebar -eq [IntPtr]::Zero) { return $shell }
            return $rebar
        }
        "task_switch" {
            $rebar = [DiagnoseTaskbarNative]::FindWindowEx($shell, [IntPtr]::Zero, "ReBarWindow32", $null)
            $taskSwitch = if ($rebar -ne [IntPtr]::Zero) {
                [DiagnoseTaskbarNative]::FindWindowEx($rebar, [IntPtr]::Zero, "MSTaskSwWClass", $null)
            } else {
                [IntPtr]::Zero
            }
            if ($taskSwitch -eq [IntPtr]::Zero) {
                $taskSwitch = [DiagnoseTaskbarNative]::FindWindowEx($shell, [IntPtr]::Zero, "MSTaskSwWClass", $null)
            }
            if ($taskSwitch -eq [IntPtr]::Zero) { return $shell }
            return $taskSwitch
        }
        "composition_bridge" {
            $bridge = [DiagnoseTaskbarNative]::FindWindowEx($shell, [IntPtr]::Zero, "Windows.UI.Composition.DesktopWindowContentBridge", $null)
            if ($bridge -eq [IntPtr]::Zero) { return $shell }
            return $bridge
        }
        default { return $shell }
    }
}

function Measure-VisualDelta {
    param(
        [string]$BeforePath,
        [string]$AfterPath,
        [object]$ShellRect,
        [object]$ModuleRect
    )

    if (-not $ModuleRect) {
        return [pscustomobject]@{
            CropValid = $false
            MeanDelta = 0.0
            BrightPixels = 0
            Width = 0
            Height = 0
        }
    }

    $relativeLeft = $ModuleRect.Left - $ShellRect.Left
    $relativeTop = $ModuleRect.Top - $ShellRect.Top
    $width = $ModuleRect.Right - $ModuleRect.Left
    $height = $ModuleRect.Bottom - $ModuleRect.Top

    $before = [System.Drawing.Bitmap]::FromFile($BeforePath)
    $after = [System.Drawing.Bitmap]::FromFile($AfterPath)
    try {
        if ($relativeLeft -lt 0 -or $relativeTop -lt 0 -or $width -le 0 -or $height -le 0) {
            return [pscustomobject]@{
                CropValid = $false
                MeanDelta = 0.0
                BrightPixels = 0
                Width = $width
                Height = $height
            }
        }

        if (($relativeLeft + $width) -gt $after.Width -or ($relativeTop + $height) -gt $after.Height) {
            return [pscustomobject]@{
                CropValid = $false
                MeanDelta = 0.0
                BrightPixels = 0
                Width = $width
                Height = $height
            }
        }

        $deltaSum = 0.0
        $bright = 0
        for ($x = 0; $x -lt $width; $x++) {
            for ($y = 0; $y -lt $height; $y++) {
                $beforePixel = $before.GetPixel($relativeLeft + $x, $relativeTop + $y)
                $afterPixel = $after.GetPixel($relativeLeft + $x, $relativeTop + $y)
                $deltaSum += [Math]::Abs($afterPixel.R - $beforePixel.R)
                $deltaSum += [Math]::Abs($afterPixel.G - $beforePixel.G)
                $deltaSum += [Math]::Abs($afterPixel.B - $beforePixel.B)
                if ($afterPixel.R -gt 220 -and $afterPixel.G -gt 220 -and $afterPixel.B -gt 220) {
                    $bright++
                }
            }
        }

        $meanDelta = $deltaSum / ([Math]::Max(1, $width * $height * 3))
        return [pscustomobject]@{
            CropValid = $true
            MeanDelta = [Math]::Round($meanDelta, 2)
            BrightPixels = $bright
            Width = $width
            Height = $height
        }
    } finally {
        $before.Dispose()
        $after.Dispose()
    }
}

function Invoke-Variant {
    param(
        [string]$Parent,
        [string]$Anchor,
        [string]$CoordMode
    )

    $variantId = "{0}-{1}-{2}" -f $Parent, $Anchor, $CoordMode
    $styleMode = if ([string]::IsNullOrWhiteSpace($env:TASKBAR_MVP_STYLE_MODE)) { "child" } else { $env:TASKBAR_MVP_STYLE_MODE }
    $refreshMode = if ([string]::IsNullOrWhiteSpace($env:TASKBAR_MVP_REFRESH_MODE)) { "none" } else { $env:TASKBAR_MVP_REFRESH_MODE }
    $diagPath = Join-Path $OutDir "$variantId.json"
    $stdoutPath = Join-Path $OutDir "$variantId.stdout.log"
    $stderrPath = Join-Path $OutDir "$variantId.stderr.log"
    $beforePath = Join-Path $OutDir "$variantId.before.png"
    $afterPath = Join-Path $OutDir "$variantId.after.png"
    $childPath = Join-Path $OutDir "$variantId.child.png"
    $screenBeforePath = Join-Path $OutDir "$variantId.screen.before.png"
    $screenAfterPath = Join-Path $OutDir "$variantId.screen.after.png"

    if (Test-Path $diagPath) { Remove-Item -LiteralPath $diagPath -Force }

    Clear-StaleTaskbarChildren
    $captureWindow = Get-CandidateParentHandle -Parent $Parent
    $beforeCapture = Get-TaskbarCapture -Window $captureWindow -Path $beforePath
    $beforeScreenCapture = Get-ScreenCapture -Rect $beforeCapture.Rect -Path $screenBeforePath

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = New-Object System.Diagnostics.ProcessStartInfo($exe)
    $proc.StartInfo.WorkingDirectory = Split-Path -Parent $exe
    $proc.StartInfo.UseShellExecute = $false
    $proc.StartInfo.RedirectStandardOutput = $true
    $proc.StartInfo.RedirectStandardError = $true
    $proc.StartInfo.EnvironmentVariables["TASKBAR_MVP_PARENT"] = $Parent
    $proc.StartInfo.EnvironmentVariables["TASKBAR_MVP_ANCHOR"] = $Anchor
    $proc.StartInfo.EnvironmentVariables["TASKBAR_MVP_COORD_MODE"] = $CoordMode
    $proc.StartInfo.EnvironmentVariables["TASKBAR_MVP_STYLE_MODE"] = $styleMode
    $proc.StartInfo.EnvironmentVariables["TASKBAR_MVP_REFRESH_MODE"] = $refreshMode
    $proc.StartInfo.EnvironmentVariables["TASKBAR_MVP_DIAG_FILE"] = $diagPath
    $null = $proc.Start()

    $deadline = (Get-Date).AddSeconds(4)
    while ((Get-Date) -lt $deadline -and -not (Test-Path $diagPath)) {
        Start-Sleep -Milliseconds 100
    }

    Start-Sleep -Milliseconds 500
    $afterCapture = Get-TaskbarCapture -Window $captureWindow -Path $afterPath

    if (Test-Path $diagPath) {
        $diag = Get-Content -Raw $diagPath | ConvertFrom-Json
    } else {
        $diag = $null
    }

    $moduleRect = if ($diag) { Convert-RectString $diag.layout.module_rect } else { $null }
    $parentRect = if ($diag) { Convert-RectString $diag.layout.parent_rect } else { $beforeCapture.Rect }
    $afterScreenCapture = Get-ScreenCapture -Rect $parentRect -Path $screenAfterPath
    $printStats = Measure-VisualDelta -BeforePath $beforePath -AfterPath $afterPath -ShellRect $beforeCapture.Rect -ModuleRect $moduleRect
    $screenStats = Measure-VisualDelta -BeforePath $screenBeforePath -AfterPath $screenAfterPath -ShellRect $beforeScreenCapture.Rect -ModuleRect $moduleRect

    $childBrightPixels = 0
    if ($diag) {
        $child = Convert-HexHandle $diag.window.hwnd
        if ($child -ne [IntPtr]::Zero) {
            $childCapture = Get-TaskbarCapture -Window $child -Path $childPath
            $childRect = [pscustomobject]@{
                Left = $childCapture.Rect.Left
                Top = $childCapture.Rect.Top
                Right = $childCapture.Rect.Right
                Bottom = $childCapture.Rect.Bottom
            }
            $childStats = Measure-VisualDelta -BeforePath $childPath -AfterPath $childPath -ShellRect $childRect -ModuleRect $childRect
            $childBrightPixels = $childStats.BrightPixels
        }
    }

    if ($diag) {
        $child = Convert-HexHandle $diag.window.hwnd
        if ($child -ne [IntPtr]::Zero) {
            $null = [DiagnoseTaskbarNative]::SendMessage($child, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
        }
    }

    if (-not $proc.WaitForExit(3000)) {
        $proc.Kill()
        $proc.WaitForExit()
    }
    Clear-StaleTaskbarChildren

    $stdout = $proc.StandardOutput.ReadToEnd()
    $stderr = $proc.StandardError.ReadToEnd()
    Set-Content -LiteralPath $stdoutPath -Value $stdout
    Set-Content -LiteralPath $stderrPath -Value $stderr

    $attachSuccess = $false
    $layoutMoved = $false
    if ($diag) {
        $attachSuccess = [bool]$diag.attach.success
        $layoutMoved = [bool]$diag.layout.moved
    }

    $withinParent = $false
    if ($moduleRect -and $parentRect) {
        $withinParent =
            $moduleRect.Left -ge $parentRect.Left -and
            $moduleRect.Top -ge $parentRect.Top -and
            $moduleRect.Right -le $parentRect.Right -and
            $moduleRect.Bottom -le $parentRect.Bottom
    }

    $printVisualPass = $attachSuccess -and $layoutMoved -and $withinParent -and $printStats.CropValid -and $printStats.MeanDelta -gt 8 -and $printStats.BrightPixels -gt 20
    $visualPass = $attachSuccess -and $layoutMoved -and $withinParent -and $screenStats.CropValid -and $screenStats.MeanDelta -gt 8 -and $screenStats.BrightPixels -gt 20
    $renderPass = $childBrightPixels -gt 20
    $pass = $visualPass -and $renderPass

    [pscustomobject]@{
        Variant = $variantId
        Parent = $Parent
        Anchor = $Anchor
        CoordMode = $CoordMode
        StyleMode = if ($diag) { $diag.config.style_mode } else { $styleMode }
        RefreshMode = if ($diag) { $diag.config.refresh_mode } else { $refreshMode }
        AttachSuccess = $attachSuccess
        LayoutMoved = $layoutMoved
        WithinParent = $withinParent
        CropValid = $screenStats.CropValid
        MeanDelta = $screenStats.MeanDelta
        BrightPixels = $screenStats.BrightPixels
        ChildBrightPixels = $childBrightPixels
        VisualPass = $visualPass
        PrintCropValid = $printStats.CropValid
        PrintMeanDelta = $printStats.MeanDelta
        PrintBrightPixels = $printStats.BrightPixels
        PrintVisualPass = $printVisualPass
        RenderPass = $renderPass
        ModuleRect = if ($diag) { $diag.layout.module_rect } else { "" }
        ParentRect = if ($diag) { $diag.layout.parent_rect } else { "" }
        CandidateParent = if ($diag) { $diag.probe.candidate_parent } else { "" }
        CurrentParent = if ($diag) { $diag.attach.current_parent } else { "" }
        ExitCode = $proc.ExitCode
        Pass = $pass
        Diagnostics = $diagPath
        BeforeCapture = $beforePath
        AfterCapture = $afterPath
        ChildCapture = $childPath
        ScreenBeforeCapture = $screenBeforePath
        ScreenAfterCapture = $screenAfterPath
        StdoutLog = $stdoutPath
        StderrLog = $stderrPath
    }
}

$results = foreach ($parent in $Parents) {
    foreach ($anchor in $Anchors) {
        foreach ($coordMode in $CoordModes) {
            Invoke-Variant -Parent $parent -Anchor $anchor -CoordMode $coordMode
        }
    }
}

$summaryPath = Join-Path $OutDir "summary.json"
$results | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $summaryPath
$results |
    Sort-Object @{Expression="Pass";Descending=$true}, @{Expression="VisualPass";Descending=$true}, @{Expression="MeanDelta";Descending=$true} |
    Format-Table Variant,StyleMode,RefreshMode,Pass,VisualPass,PrintVisualPass,RenderPass,WithinParent,AttachSuccess,LayoutMoved,MeanDelta,PrintMeanDelta,BrightPixels,ChildBrightPixels,ModuleRect -AutoSize
Write-Host ""
Write-Host "Summary JSON: $summaryPath"

if (-not ($results | Where-Object { $_.Pass })) {
    exit 1
}
