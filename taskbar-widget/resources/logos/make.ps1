Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap(16,16)
$bmp.SetPixel(0,0,[System.Drawing.Color]::FromArgb(255,255,0,0))
$bmp.Save('D:\project\cc-traffic-light\taskbar-widget\resources\logos\test.png', [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "saved"
