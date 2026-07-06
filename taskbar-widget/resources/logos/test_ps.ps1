# Try to create a simple PNG using .NET
try {
    # Try loading System.Drawing
    [System.Reflection.Assembly]::LoadWithPartialName("System.Drawing") | Out-Null
    $bmp = New-Object System.Drawing.Bitmap(16,16)
    $bmp.SetPixel(0,0,[System.Drawing.Color]::FromArgb(255,255,0,0))
    $bmp.Save("D:\project\cc-traffic-light\taskbar-widget\resources\logos\from_ps.png", [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    "SUCCESS: System.Drawing worked"
}
catch {
    "FAILED System.Drawing: $_"
}

# Also try using WPF
try {
    Add-Type -AssemblyName PresentationCore
    $wbm = New-Object System.Windows.Media.Imaging.WriteableBitmap(16,16,96,96,System.Windows.Media.PixelFormats.Bgra32,$null)
    "SUCCESS: WPF worked"
}
catch {
    "FAILED WPF: $_"
}
