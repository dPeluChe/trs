#Requires -Version 5.1
# trs installer for Windows: downloads the prebuilt binary.
#
# Usage:
#   irm https://usetrs.dev/install.ps1 | iex
#   # or the raw GitHub URL (equivalent, works before DNS propagates):
#   irm https://raw.githubusercontent.com/dPeluChe/trs/main/docs/install.ps1 | iex
#
# Options (env vars):
#   $env:TRS_VERSION          pin a specific release (default: latest)
#   $env:TRS_INSTALL_DIR      override install location
#   $env:TRS_NO_MODIFY_PATH=1  skip auto-adding the install dir to User PATH
#
# Default install dir: $USERPROFILE\.local\bin (XDG parity with Unix). When
# it's not already in User PATH, we add it via SetEnvironmentVariable so the
# install works in the next terminal session with zero manual steps.

$ErrorActionPreference = "Stop"

$Repo = "dPeluChe/trs"
$BinName = "trs.exe"

function Write-Info    { param($msg) Write-Host "▸ $msg" -ForegroundColor Cyan }
function Write-Ok      { param($msg) Write-Host "✓ $msg" -ForegroundColor Green }
function Write-Warning2 { param($msg) Write-Host "! $msg" -ForegroundColor Yellow }
function Write-Err     { param($msg) Write-Host "✗ $msg" -ForegroundColor Red; exit 1 }

# ------------------------------------------------------------------
# Pick install dir: prefer any $PATH candidate already set up, fall back to
# the XDG-on-Windows default. User override via $env:TRS_INSTALL_DIR wins.
# ------------------------------------------------------------------

function Get-PathEntries {
    param($Scope) # 'User' or 'Process'
    $raw = [Environment]::GetEnvironmentVariable("Path", $Scope)
    if (-not $raw) { return @() }
    return $raw -split ";" | Where-Object { $_ }
}

function Test-InUserPath {
    param($Dir)
    (Get-PathEntries 'User')  -contains $Dir -or
    (Get-PathEntries 'Process') -contains $Dir
}

function Get-InstallDir {
    if ($env:TRS_INSTALL_DIR) { return $env:TRS_INSTALL_DIR }
    $candidates = @(
        (Join-Path $env:USERPROFILE ".local\bin"),
        (Join-Path $env:USERPROFILE "bin")
    )
    foreach ($d in $candidates) {
        if (Test-InUserPath $d) { return $d }
    }
    # Nothing pre-configured: default to XDG convention. We'll add it to
    # User PATH automatically below.
    return $candidates[0]
}

$InstallDir = Get-InstallDir

# ------------------------------------------------------------------
# Detect arch + resolve version
# ------------------------------------------------------------------

$arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    'X64'   { 'x64' }
    'Arm64' { 'arm64' }
    default { Write-Err "unsupported arch: $_" }
}
$platform = "win32-$arch"

if ($env:TRS_VERSION) {
    $version = $env:TRS_VERSION
} else {
    # Unauthenticated callers get 60 API requests an hour per IP, and
    # Invoke-RestMethod throws on the 403 rather than returning empty, so the
    # user saw a PowerShell stack trace instead of our message. Catch it.
    $version = $null
    try {
        $version = (Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest").tag_name
    } catch {
        $version = $null
    }
    if (-not $version) {
        Write-Err "could not resolve the latest release. Retry, or pin one from https://github.com/$Repo/releases with `$env:TRS_VERSION='vX.Y.Z'"
    }
}

$asset = "trs-windows-x64.exe"  # only x64 builds today; adjust when arm64 is published
$url = "https://github.com/$Repo/releases/download/$version/$asset"

Write-Host "`ntrs installer" -ForegroundColor White
Write-Host "https://github.com/$Repo`n" -ForegroundColor DarkGray
Write-Info "platform: $platform"
Write-Info "version:  $version"
Write-Info "url:      $url"
Write-Info "install:  $InstallDir\$BinName`n"

# ------------------------------------------------------------------
# Download
# ------------------------------------------------------------------

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$target = Join-Path $InstallDir $BinName

Write-Info "downloading..."
try {
    Invoke-WebRequest -Uri $url -OutFile $target -UseBasicParsing
} catch {
    Write-Err "download failed: $_"
}

# Verify checksum against SHA256SUMS published in the release.
# Optional for releases < v0.6.3 (no sums file shipped); required when
# the file exists. Mismatch hard-fails the install.
$sumsUrl = "https://github.com/$Repo/releases/download/$version/SHA256SUMS"
$sumsTmp = Join-Path $env:TEMP "trs-sums.txt"
try {
    Invoke-WebRequest -Uri $sumsUrl -OutFile $sumsTmp -UseBasicParsing -ErrorAction Stop
    $line = Get-Content $sumsTmp | Where-Object { $_ -match " $asset$" } | Select-Object -First 1
    if (-not $line) {
        Write-Err "SHA256SUMS published for $version but missing entry for $asset"
    }
    $expected = ($line -split '\s+')[0]
    $actual = (Get-FileHash -Algorithm SHA256 $target).Hash.ToLower()
    if ($expected -ne $actual) {
        Write-Err "checksum mismatch for $asset (expected $expected, got $actual)"
    }
    Write-Ok "sha256 verified"
} catch {
    Write-Info "no SHA256SUMS for $version, skipping checksum (older release)"
} finally {
    if (Test-Path $sumsTmp) { Remove-Item $sumsTmp -Force }
}

# ------------------------------------------------------------------
# Detect existing install (npm / choco / scoop / manual)
# ------------------------------------------------------------------

$existing = Get-Command trs -ErrorAction SilentlyContinue
if ($existing -and $existing.Source -ne $target) {
    Write-Warning2 "Another trs is already installed at:"
    Write-Host "       $($existing.Source)"
    Write-Host "       PATH order decides which runs. Put $InstallDir first to prefer this install.`n"
}

Write-Ok "installed trs $version to $target"

# ------------------------------------------------------------------
# PATH handling: auto-add to User PATH unless opted out
# ------------------------------------------------------------------

if (Test-InUserPath $InstallDir) {
    Write-Ok "$InstallDir is in PATH"
    Write-Host "`nDone. " -NoNewline -ForegroundColor Green
    Write-Host "Run: " -NoNewline
    Write-Host "trs doctor" -ForegroundColor Cyan
    Write-Host ""
} elseif ($env:TRS_NO_MODIFY_PATH) {
    Write-Warning2 "$InstallDir is not in your PATH (TRS_NO_MODIFY_PATH set, not modifying)"
    Write-Host "  Add it later with:"
    Write-Host ("    [Environment]::SetEnvironmentVariable('Path', '{0};' + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')" -f $InstallDir) -ForegroundColor Cyan
    Write-Host ""
} else {
    # Add to User PATH via registry. Persists across sessions. No admin needed.
    # Idempotent: Test-InUserPath already returned false, so we know it's absent.
    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $newUserPath = if ($currentUserPath) { "$InstallDir;$currentUserPath" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
    Write-Ok "added $InstallDir to User PATH"
    Write-Host "`n" -NoNewline
    Write-Warning2 "open a new terminal session for PATH changes to take effect"
    Write-Host "  (or in this session: " -NoNewline
    Write-Host "`$env:Path = '$InstallDir;' + `$env:Path" -ForegroundColor Cyan -NoNewline
    Write-Host ")"
    Write-Host "`nDone. " -NoNewline -ForegroundColor Green
    Write-Host "After reopening your terminal, try: " -NoNewline
    Write-Host "trs doctor" -ForegroundColor Cyan
    Write-Host ""
}
