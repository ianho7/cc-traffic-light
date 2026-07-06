[System.Drawing.Bitmap]$bmp = New-Object System.Drawing.Bitmap(16,16)
$bmp.SetPixel(0,0,[System.Drawing.Color]::FromArgb(255,255,0,0))
$bmp.Save("test_output.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
