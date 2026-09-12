param(
    [switch]$Install,
    [switch]$Pack,
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$pluginId = "dev.flowingspdg.countersample.rust"
$pluginDir = Join-Path $root "$pluginId.sdPlugin"
$piDir = Join-Path $root "pi"
$repoRoot = (Resolve-Path (Join-Path $root "..\..")).Path
if (-not $OutputDir) {
    $OutputDir = Join-Path $repoRoot "artifacts\plugin"
}

function Get-HostTarget {
    if ($IsWindows -or $env:OS -like "*Windows*") {
        return "x86_64-pc-windows-msvc"
    }
    if ($IsMacOS) {
        if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -eq "Arm64") {
            return "aarch64-apple-darwin"
        }
        return "x86_64-apple-darwin"
    }
    return "x86_64-unknown-linux-gnu"
}

function Publish-Target([string]$target, [string]$ridFolder, [string]$fileName) {
    rustup target add $target | Out-Null
    cargo build -p counter-sample --release --bin counter_sample_plugin --target $target
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build --target $target failed"
    }
    $out = Join-Path $pluginDir "bin\$ridFolder"
    New-Item -ItemType Directory -Force -Path $out | Out-Null
    $built = Join-Path $repoRoot "target\$target\release"
    $srcName = if ($fileName.EndsWith(".exe")) { "counter_sample_plugin.exe" } else { "counter_sample_plugin" }
    $src = Join-Path $built $srcName
    if (-not (Test-Path $src)) {
        throw "built binary not found: $src"
    }
    Copy-Item -Force $src (Join-Path $out $fileName)
}

Push-Location $repoRoot
try {
    cargo run -p counter-sample --bin typegen
    if ($LASTEXITCODE -ne 0) {
        throw "typegen failed"
    }

    Push-Location $piDir
    if (-not (Test-Path "node_modules")) {
        npm install
    }
    npm run build
    Pop-Location

    if ($IsWindows -or $env:OS -like "*Windows*") {
        Publish-Target "x86_64-pc-windows-msvc" "win-x64" "$pluginId.exe"
    }
    foreach ($pair in @(
        @{ Target = "aarch64-apple-darwin"; Rid = "osx-arm64"; Name = $pluginId },
        @{ Target = "x86_64-apple-darwin"; Rid = "osx-x64"; Name = $pluginId }
    )) {
        if (-not $IsMacOS) {
            $placeholder = Join-Path $pluginDir "bin\$($pair.Rid)\$($pair.Name)"
            New-Item -ItemType Directory -Force -Path (Split-Path $placeholder) | Out-Null
            if (-not (Test-Path $placeholder)) {
                New-Item -ItemType File -Path $placeholder | Out-Null
            }
            Write-Host "Skipping $($pair.Target): macOS binaries are built on macOS"
            continue
        }
        try {
            Publish-Target $pair.Target $pair.Rid $pair.Name
        }
        catch {
            Write-Host "Skipping $($pair.Target): $_"
            $placeholder = Join-Path $pluginDir "bin\$($pair.Rid)\$($pair.Name)"
            New-Item -ItemType Directory -Force -Path (Split-Path $placeholder) | Out-Null
            if (-not (Test-Path $placeholder)) {
                New-Item -ItemType File -Path $placeholder | Out-Null
            }
        }
    }

    Write-Host "Published plugin bundle to $pluginDir"

    if ($Pack) {
        New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
        $zipPath = Join-Path $OutputDir "$pluginId.streamDeckPlugin"
        if (Test-Path $zipPath) {
            Remove-Item -Force $zipPath
        }
        $cli = Get-Command streamdeck -ErrorAction SilentlyContinue
        if ($cli) {
            & streamdeck pack $pluginDir --output $OutputDir --force
            if ($LASTEXITCODE -ne 0) {
                throw "streamdeck pack failed with exit code $LASTEXITCODE"
            }
        }
        else {
            Add-Type -AssemblyName System.IO.Compression.FileSystem
            [System.IO.Compression.ZipFile]::CreateFromDirectory($pluginDir, $zipPath, [System.IO.Compression.CompressionLevel]::Optimal, $true)
        }
        Write-Host "Packed plugin installer in $OutputDir"
    }

    if ($Install) {
        $destRoot = Join-Path $env:APPDATA "Elgato\StreamDeck\Plugins"
        $dest = Join-Path $destRoot "$pluginId.sdPlugin"
        New-Item -ItemType Directory -Force -Path $destRoot | Out-Null
        $cli = Get-Command streamdeck -ErrorAction SilentlyContinue
        if ($cli) {
            & streamdeck stop $pluginId
        }
        if (Test-Path $dest) {
            Remove-Item -Recurse -Force $dest
        }
        Copy-Item -Recurse -Force $pluginDir $dest
        Write-Host "Installed plugin to $dest"
        if ($cli) {
            & streamdeck restart $pluginId
        }
    }

    $global:LASTEXITCODE = 0
}
finally {
    Pop-Location
}
