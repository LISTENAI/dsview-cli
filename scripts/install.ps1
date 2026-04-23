param(
    [string]$Version = "",
    [string]$Prefix = "",
    [string]$BinDir = "",
    [string]$Repo = "",
    [switch]$Force,
    [switch]$SkipChecksum,
    [switch]$DryRun,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Repo)) {
    $Repo = if ($env:DSVIEW_REPO) { $env:DSVIEW_REPO } else { "LISTENAI/dsview-cli" }
}
if ([string]::IsNullOrWhiteSpace($Prefix)) {
    $Prefix = if ($env:DSVIEW_PREFIX) {
        $env:DSVIEW_PREFIX
    } else {
        Join-Path $env:LOCALAPPDATA "Programs\dsview-cli"
    }
}
if ([string]::IsNullOrWhiteSpace($BinDir)) {
    $BinDir = if ($env:DSVIEW_BIN_DIR) {
        $env:DSVIEW_BIN_DIR
    } else {
        Join-Path $Prefix "bin"
    }
}

function Show-Usage {
    @"
Install DSView CLI from GitHub release bundles.

Usage:
  install.ps1 [options]

Options:
  -Version <tag>      Install a specific release tag (default: latest)
  -Prefix <path>      Bundle install root (default: $Prefix)
  -BinDir <path>      Launcher directory (default: $BinDir)
  -Repo <owner/name>  GitHub repository (default: $Repo)
  -Force              Replace an existing installation for the same version
  -SkipChecksum       Skip SHA-256 verification
  -DryRun             Print the resolved install plan without downloading
  -Help               Show this help message
"@
}

function Write-Log([string]$Message) {
    Write-Host "==> $Message"
}

function Write-Warn([string]$Message) {
    Write-Warning $Message
}

function Fail([string]$Message) {
    throw $Message
}

function Require-Command([string]$Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Fail "required command not found: $Name"
    }
}

function Resolve-LatestVersion {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ Accept = "application/vnd.github+json" }
    if (-not $release.tag_name) {
        Fail "unable to determine latest release from $apiUrl"
    }
    return [string]$release.tag_name
}

function Get-TargetTriple {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        ([System.Runtime.InteropServices.Architecture]::X64) { return "x86_64-pc-windows-msvc" }
        ([System.Runtime.InteropServices.Architecture]::Arm64) { return "aarch64-pc-windows-msvc" }
        default { Fail "unsupported Windows architecture: $arch" }
    }
}

function Verify-Checksum([string]$ArchivePath, [string]$ChecksumPath, [string]$AssetName) {
    $expected = $null
    foreach ($line in Get-Content -LiteralPath $ChecksumPath) {
        if ($line -match "^\s*([0-9a-fA-F]{64})\s+(.+?)\s*$") {
            $entry = [System.IO.Path]::GetFileName($matches[2])
            if ($entry -eq $AssetName) {
                $expected = $matches[1].ToLowerInvariant()
                break
            }
        }
    }
    if (-not $expected) {
        Fail "checksum entry not found for $AssetName"
    }

    $actual = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        Fail "checksum mismatch for $AssetName"
    }
}

function Normalize-PathEntry([string]$PathEntry) {
    if ([string]::IsNullOrWhiteSpace($PathEntry)) {
        return ""
    }

    return $PathEntry.Trim().Trim('"').TrimEnd('\', '/')
}

function Ensure-UserPathContains([string]$PathEntry) {
    $normalizedTarget = Normalize-PathEntry $PathEntry
    if ([string]::IsNullOrWhiteSpace($normalizedTarget)) {
        return $false
    }

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $entries = @()
    if (-not [string]::IsNullOrWhiteSpace($userPath)) {
        $entries = $userPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    }

    foreach ($entry in $entries) {
        if ((Normalize-PathEntry $entry) -ieq $normalizedTarget) {
            return $false
        }
    }

    [Environment]::SetEnvironmentVariable("Path", (($entries + $PathEntry) -join ';'), "User")
    return $true
}

function Write-Launcher([string]$LauncherPath, [string]$InstallDir) {
    $content = @"
@echo off
setlocal
set "DSVIEW_ROOT=$InstallDir"
set "PATH=%DSVIEW_ROOT%;%DSVIEW_ROOT%\runtime;%PATH%"
"%DSVIEW_ROOT%\dsview-cli.exe" %*
set EXITCODE=%ERRORLEVEL%
endlocal & exit /b %EXITCODE%
"@
    Set-Content -LiteralPath $LauncherPath -Value $content -Encoding ASCII
}

if ($Help) {
    Show-Usage
    exit 0
}

Require-Command "tar.exe"

$target = Get-TargetTriple
if ([string]::IsNullOrWhiteSpace($Version)) {
    Write-Log "Resolving latest release for $Repo"
    $Version = Resolve-LatestVersion
}

$assetName = "dsview-cli-$Version-$target.tar.gz"
$checksumName = "dsview-cli-$Version-SHA256SUMS.txt"
$releaseBaseUrl = "https://github.com/$Repo/releases/download/$Version"
$archiveUrl = "$releaseBaseUrl/$assetName"
$checksumUrl = "$releaseBaseUrl/$checksumName"
$installDir = Join-Path $Prefix $Version
$launcherPath = Join-Path $BinDir "dsview-cli.cmd"

if ($DryRun) {
    Write-Output "repo=$Repo"
    Write-Output "version=$Version"
    Write-Output "target=$target"
    Write-Output "archive_url=$archiveUrl"
    Write-Output "checksum_url=$checksumUrl"
    Write-Output "install_dir=$installDir"
    Write-Output "launcher_path=$launcherPath"
    exit 0
}

if ((Test-Path -LiteralPath $installDir) -and (-not $Force)) {
    Fail "installation already exists at $installDir (use -Force to replace it)"
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("dsview-cli-install-" + [System.Guid]::NewGuid().ToString("N"))
$extractDir = Join-Path $tempRoot "extract"
$archivePath = Join-Path $tempRoot $assetName
$checksumPath = Join-Path $tempRoot $checksumName
$bundleRoot = Join-Path $extractDir ("dsview-cli-$Version-$target")

try {
    New-Item -ItemType Directory -Force -Path $Prefix | Out-Null
    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    New-Item -ItemType Directory -Force -Path $extractDir | Out-Null

    Write-Log "Downloading $assetName"
    Invoke-WebRequest -Uri $archiveUrl -OutFile $archivePath

    if (-not $SkipChecksum) {
        Write-Log "Downloading checksum file"
        Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath
        Write-Log "Verifying checksum"
        Verify-Checksum -ArchivePath $archivePath -ChecksumPath $checksumPath -AssetName $assetName
    } else {
        Write-Warn "Skipping checksum verification"
    }

    if (Test-Path -LiteralPath $installDir) {
        Remove-Item -LiteralPath $installDir -Recurse -Force
    }

    Write-Log "Extracting release bundle"
    & tar.exe -xzf $archivePath -C $extractDir
    if ($LASTEXITCODE -ne 0) {
        Fail "failed to extract $archivePath"
    }
    if (-not (Test-Path -LiteralPath $bundleRoot)) {
        Fail "expected extracted bundle directory missing: $bundleRoot"
    }

    Write-Log "Installing bundle to $installDir"
    Move-Item -LiteralPath $bundleRoot -Destination $installDir

    Write-Log "Writing launcher to $launcherPath"
    Write-Launcher -LauncherPath $launcherPath -InstallDir $installDir

    Write-Log "Running smoke checks"
    & $launcherPath --version | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Fail "smoke check failed: dsview-cli --version"
    }
    & $launcherPath devices list --help | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Fail "smoke check failed: dsview-cli devices list --help"
    }

    Write-Log "Installed DSView CLI $Version for $target"
    Write-Log "Run: $launcherPath --help"
    if (Ensure-UserPathContains -PathEntry $BinDir) {
        Write-Log "Added $BinDir to your user PATH"
    } else {
        Write-Log "$BinDir is already present in your user PATH"
    }
    Write-Warn "Restart your shell to reload PATH changes before invoking dsview-cli.cmd without a full path."
} finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
