param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$pluginOut = Join-Path $root "artifacts\plugin"
New-Item -ItemType Directory -Force -Path $pluginOut | Out-Null
& (Join-Path $root "samples\counter-sample\publish.ps1") -Pack -OutputDir $pluginOut
Write-Host "Plugin package: $pluginOut"
