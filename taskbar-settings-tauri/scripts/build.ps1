$ErrorActionPreference = "Stop"

$nodePath = if ($env:npm_node_execpath) {
    $env:npm_node_execpath
}
else {
    (Get-Command node -ErrorAction Stop).Source
}

& $nodePath .\node_modules\typescript\lib\tsc.js -b
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

& $nodePath .\node_modules\vite\bin\vite.js build
exit $LASTEXITCODE
