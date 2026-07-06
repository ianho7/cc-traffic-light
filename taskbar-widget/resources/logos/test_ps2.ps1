$errFile = "D:\project\cc-traffic-light\taskbar-widget\resources\logos\error_log.txt"
"Starting script" | Out-File $errFile

try {
    # Check what assemblies are loaded
    [System.AppDomain]::CurrentDomain.GetAssemblies() | ForEach-Object { $_.FullName } | Out-File $errFile -Append
} catch {
    "Error getting assemblies: $_" | Out-File $errFile -Append
}

try {
    $bmp = New-Object System.Drawing.Bitmap(16,16)
    "Bitmap created" | Out-File $errFile -Append
    $bmp.SetPixel(0,0,[System.Drawing.Color]::FromArgb(255,255,0,0))
    $bmp.Save("D:\project\cc-traffic-light\taskbar-widget\resources\logos\from_ps.png", [System.Drawing.Imaging.ImageFormat]::Png)
    "Saved" | Out-File $errFile -Append
    $bmp.Dispose()
} catch {
    "Error: $_" | Out-File $errFile -Append
    $_.Exception | Out-File $errFile -Append
}
