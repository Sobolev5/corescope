param()
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
cargo build --release --manifest-path (Join-Path $root 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
$destination=Join-Path $root 'dist'
New-Item -ItemType Directory -Force $destination | Out-Null
Copy-Item -Path (Join-Path $root 'assets\hardware') -Destination $destination -Recurse -Force
Copy-Item -LiteralPath (Join-Path $root 'target\release\corescope.exe') -Destination (Join-Path $destination 'corescope.exe')
Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination (Join-Path $destination 'README.md')
Copy-Item -LiteralPath (Join-Path $root 'assets\webview2\WebView2Loader.dll') -Destination (Join-Path $destination 'WebView2Loader.dll')
Copy-Item -LiteralPath (Join-Path $root 'assets\webview2\LICENSE.txt') -Destination (Join-Path $destination 'WebView2-LICENSE.txt')
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination (Join-Path $destination 'LICENSE')
Write-Output "Portable build: $destination"
