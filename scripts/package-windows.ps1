$ErrorActionPreference = 'Stop'
$taskRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
Set-Location -LiteralPath $taskRoot
$artifactRoot = Join-Path $taskRoot 'output\windows'
$portableRoot = Join-Path $artifactRoot 'portable'
New-Item -ItemType Directory -Force -Path $portableRoot | Out-Null
Copy-Item -LiteralPath (Join-Path $taskRoot 'src-tauri\target\release\project-arena.exe') -Destination (Join-Path $portableRoot 'Project-ARENA.exe') -Force
foreach ($name in @('LICENSE','THIRD_PARTY_NOTICES.md')) { Copy-Item -LiteralPath (Join-Path $taskRoot $name) -Destination $portableRoot -Force }
Copy-Item -LiteralPath (Join-Path $taskRoot 'build-notices\THIRD-PARTY-LICENSES.txt') -Destination $portableRoot -Force
Copy-Item -LiteralPath (Join-Path $taskRoot 'src-tauri\target\release\bundle\nsis\Project ARENA_0.1.2_x64-setup.exe') -Destination $artifactRoot -Force
Compress-Archive -Path (Join-Path $portableRoot '*') -DestinationPath (Join-Path $artifactRoot 'Project-ARENA-0.1.2-portable.zip') -Force
$sourceHash = (Get-FileHash -LiteralPath (Join-Path $taskRoot 'src-tauri\target\release\project-arena.exe')).Hash
$copyHash = (Get-FileHash -LiteralPath (Join-Path $portableRoot 'Project-ARENA.exe')).Hash
if ($sourceHash -ne $copyHash) { throw 'Portable executable does not match the built binary' }
$paths = @((Join-Path $portableRoot 'Project-ARENA.exe'),(Join-Path $artifactRoot 'Project-ARENA-0.1.2-portable.zip'),(Join-Path $artifactRoot 'Project ARENA_0.1.2_x64-setup.exe'))
$manifest = foreach ($path in $paths) { $item=Get-Item -LiteralPath $path; @{path=$path;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $path).Hash} }
$manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $artifactRoot 'artifact-manifest.json') -Encoding utf8
$manifest | ConvertTo-Json
