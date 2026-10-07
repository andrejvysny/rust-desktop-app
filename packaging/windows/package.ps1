$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '../..')
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
$staging = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
try {
    New-Item -ItemType Directory -Path $staging | Out-Null
    New-Item -ItemType Directory -Path 'dist' -Force | Out-Null
    Copy-Item (Join-Path $targetDir 'release/app.exe') $staging
    Copy-Item (Join-Path $targetDir 'release/app-cli.exe') $staging
    Copy-Item 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'ATTRIBUTION.md' $staging
    Compress-Archive -Path (Join-Path $staging '*') -DestinationPath 'dist/app-template-windows.zip' -Force
} finally {
    Remove-Item $staging -Recurse -Force
}
