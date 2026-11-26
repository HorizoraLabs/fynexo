# Fynexo Windows Release Packager
# Builds release binary, creates portable ZIP, and builds Inno Setup installer if ISCC is available.
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Push-Location $Root
try {
    # Read version from Cargo.toml
    $cargoToml = Get-Content -Raw (Join-Path $Root "Cargo.toml")
    $verMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
    $Version = if ($verMatch.Success) { $verMatch.Groups[1].Value } else { "0.1.0" }

    Write-Host "[Fynexo] Packaging version v$Version..."

    if (-not $SkipBuild) {
        Write-Host "[Fynexo] Running cargo build --release..."
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build --release failed with exit code $LASTEXITCODE"
        }
    }

    $ExePath = Join-Path $Root "target\release\fynexo.exe"
    if (-not (Test-Path $ExePath)) {
        throw "Release binary not found at $ExePath"
    }

    $DistDir = Join-Path $Root "dist"
    $StageDir = Join-Path $DistDir "Fynexo-v$Version-win64"
    if (Test-Path $StageDir) { Remove-Item -Recurse -Force $StageDir }
    New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

    Copy-Item $ExePath (Join-Path $StageDir "fynexo.exe")
    if (Test-Path (Join-Path $Root "README.md")) {
        Copy-Item (Join-Path $Root "README.md") (Join-Path $StageDir "README.md")
    }
    if (Test-Path (Join-Path $Root "LICENSE")) {
        Copy-Item (Join-Path $Root "LICENSE") (Join-Path $StageDir "LICENSE")
    }

    $ZipPath = Join-Path $DistDir "Fynexo-v$Version-win64-portable.zip"
    if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
    Compress-Archive -Path (Join-Path $StageDir "*") -DestinationPath $ZipPath -CompressionLevel Optimal
    Write-Host "[Fynexo] Created portable archive: $ZipPath"

    $IssPath = Join-Path $Root "packaging\fynexo.iss"
    $IsccCmd = Get-Command "iscc" -ErrorAction SilentlyContinue
    if (-not $IsccCmd) {
        $DefaultIscc = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
        if (Test-Path $DefaultIscc) { $IsccCmd = Get-Item $DefaultIscc }
    }

    if ($IsccCmd -and (Test-Path $IssPath)) {
        Write-Host "[Fynexo] Building Inno Setup installer..."
        & $IsccCmd.FullName "/DMyAppVersion=$Version" $IssPath
        if ($LASTEXITCODE -ne 0) {
            throw "ISCC failed with exit code $LASTEXITCODE"
        }
        Write-Host "[Fynexo] Installer created in $DistDir"
    } else {
        Write-Host "[Fynexo] Inno Setup (iscc) not found; skipped .exe installer (portable ZIP is ready)."
        Write-Host "[Fynexo] Install Inno Setup 6 and re-run scripts\package_windows.ps1 to build the .exe installer."
    }
} finally {
    Pop-Location
}
