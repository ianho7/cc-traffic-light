function New-PngCircle {
    param([string]$Path, [byte]$R, [byte]$G, [byte]$B)
    
    # Create raw pixel data (top-left to bottom-right, BGRA)
    $pixels = [System.Collections.ArrayList]@()
    for ($y = 0; $y -lt 16; $y++) {
        for ($x = 0; $x -lt 16; $x++) {
            $cx = $x - 7.5
            $cy = $y - 7.5
            if ([Math]::Sqrt($cx*$cx + $cy*$cy) -le 7.5) {
                # Inside circle - BGRA
                $null = $pixels.Add($B)  # Blue
                $null = $pixels.Add($G)  # Green
                $null = $pixels.Add($R)  # Red
                $null = $pixels.Add(255)  # Alpha
            } else {
                # Transparent
                $null = $pixels.Add(0)   # B
                $null = $pixels.Add(0)   # G
                $null = $pixels.Add(0)   # R
                $null = $pixels.Add(0)   # A
            }
        }
    }
    
    $rawData = $pixels.ToArray()
    
    # PNG structure
    # We'll write raw uncompressed data with filter byte 0 at start of each row
    $rowSize = 1 + 16 * 4  # filter byte + 16 pixels * 4 channels
    $rawRows = [byte[]]::new($rowSize * 16)
    for ($r = 0; $r -lt 16; $r++) {
        $rawRows[$r * $rowSize] = 0  # No filter
        for ($c = 0; $c -lt 16 * 4; $c++) {
            $rawRows[$r * $rowSize + 1 + $c] = $rawData[$r * 16 * 4 + $c]
        }
    }
    
    # Use a simple approach: write raw deflate stream (no header/checksum for simplicity)
    # Actually, let's use .NET to compress
    try {
        Add-Type -AssemblyName System.IO.Compression
        $msIn = New-Object System.IO.MemoryStream($rawRows)
        $msOut = New-Object System.IO.MemoryStream
        $deflate = New-Object System.IO.Compression.DeflateStream($msOut, [System.IO.Compression.CompressionMode]::Compress)
        $msIn.CopyTo($deflate)
        $deflate.Close()
        $compressed = $msOut.ToArray()
        $msIn.Dispose()
        $msOut.Dispose()
    } catch {
        Write-Output "Compression failed: $_"
        return
    }
    
    # Build PNG file
    $pngStream = New-Object System.IO.MemoryStream
    
    # Signature
    $sig = [byte[]]@(137, 80, 78, 71, 13, 10, 26, 10)
    $pngStream.Write($sig, 0, $sig.Length)
    
    # IHDR chunk
    $ihdrData = [byte[]]@(
        0,0,0,16,  # width = 16
        0,0,0,16,  # height = 16
        8,         # bit depth = 8
        6,         # color type = RGBA
        0, 0, 0,   # compression, filter, interlace
        0, 0, 0, 0 # CRC placeholder
    )
    $ihdrLen = [byte[]]@(0,0,0,13)  # chunk length = 13
    $pngStream.Write($ihdrLen, 0, 4)
    $pngStream.Write([byte[]]@(73,72,68,82), 0, 4)  # "IHDR"
    $pngStream.Write($ihdrData, 0, $ihdrData.Length)
    
    # Calculate CRC for IHDR
    $crcData = New-Object System.Collections.ArrayList
    $null = $crcData.AddRange([byte[]]@(73,72,68,82))
    $null = $crcData.AddRange($ihdrData)
    $crcBytes = $crcData.ToArray()
    
    $crcVal = [System.IO.Compression.Crc32]::ComputeHash($crcBytes)
    # ... this is getting too complex. Let me try a simpler approach.
    
    $pngStream.Dispose()
    Write-Output "Failed to build PNG manually"
}

New-PngCircle -Path "D:\project\cc-traffic-light\taskbar-widget\resources\logos\test2.png" -R 255 -G 0 -B 0
