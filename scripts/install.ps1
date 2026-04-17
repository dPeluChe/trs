#Requires -Version 5.1
# trs installer for Windows — downloads the prebuilt binary.
#
# Usage:
#   irm https://raw.githubusercontent.com/dPeluChe/trs/main/scripts/install.ps1 | iex
#
# Options (env vars):
#   $env:TRS_VERSION     — pin a specific release (default: latest)
#   $env:TRS_INSTALL_DIR — install location (default: $env:USERPROFILE\.trs\bin)

$ErrorActionPreference = "Stop"

$Repo = "dPeluChe/trs"
$InstallDir = if ($env:TRS_INSTALL_DIR) { $env:TRS_INSTALL_DIR } else { Join-Path $env:USERPROFILE ".trs\bin" }
$BinName = "trs.exe"

function Write-Info    { param($msg) Write-Host "▸ $msg" -ForegroundColor Cyan }
function Write-Ok      { param($msg) Write-Host "✓ $msg" -ForegroundColor Green }
function Write-Warning2 { param($msg) Write-Host "! $msg" -ForegroundColor Yellow }
function Write-Err     { param($msg) Write-Host "✗ $msg" -ForegroundColor Red; exit 1 }

# --- Detect arch ---
$arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    'X64'   { 'x64' }
    'Arm64' { 'arm64' }
    default { Write-Err "unsupported arch: $_" }
}
$platform = "win32-$arch"

# --- Resolve version ---
if ($env:TRS_VERSION) {
    $version = $env:TRS_VERSION
} else {
    $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $version = $releaseInfo.tag_name
    if (-not $version) { Write-Err "could not resolve latest release (set `$env:TRS_VERSION)" }
}

# --- URLs ---
$asset = "trs-windows-x64.exe"  # only x64 builds today; adjust when arm64 is published
$url = "https://github.com/$Repo/releases/download/$version/$asset"

Write-Host "`ntrs installer" -ForegroundColor White
Write-Host "https://github.com/$Repo`n" -ForegroundColor DarkGray
Write-Info "platform: $platform"
Write-Info "version:  $version"
Write-Info "url:      $url"
Write-Info "install:  $InstallDir\$BinName`n"

# --- Download ---
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$target = Join-Path $InstallDir $BinName

Write-Info "downloading..."
try {
    Invoke-WebRequest -Uri $url -OutFile $target -UseBasicParsing
} catch {
    Write-Err "download failed: $_"
}

# --- Detect existing install (npm / choco / scoop / manual) ---
$existing = Get-Command trs -ErrorAction SilentlyContinue
if ($existing -and $existing.Source -ne $target) {
    Write-Warning2 "Another trs is already installed at:"
    Write-Host "       $($existing.Source)"
    Write-Host "       PATH order decides which runs. Put $InstallDir first to prefer this install.`n"
}

# --- PATH hint ---
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    Write-Warning2 "$InstallDir is not in your PATH"
    Write-Host "  Add it with:"
    Write-Host ("    [Environment]::SetEnvironmentVariable('Path', '{0};' + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')" -f $InstallDir) -ForegroundColor Cyan
    Write-Host "  Then restart your terminal.`n"
} else {
    Write-Ok "$InstallDir is already in PATH"
}

Write-Ok "installed trs $version to $target"
Write-Host "`nDone. Try: " -NoNewline
Write-Host "trs doctor" -ForegroundColor Cyan
Write-Host ""
