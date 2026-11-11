param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')]
    [string]$Target,
    [ValidateSet('x64-windows-release', 'arm64-windows-release')]
    [string]$HostTriplet
)

$ErrorActionPreference = 'Stop'

if ($env:VCPKG_DEFAULT_BINARY_CACHE) {
  New-Item -ItemType Directory -Force -Path $env:VCPKG_DEFAULT_BINARY_CACHE | Out-Null
}
if ($env:VCPKG_DOWNLOADS) {
  New-Item -ItemType Directory -Force -Path $env:VCPKG_DOWNLOADS | Out-Null
}

$Triplet = switch ("$Target") {
  "x86_64-pc-windows-msvc" { "x64-windows-release" }
  "aarch64-pc-windows-msvc" { "arm64-windows-release" }
  default { throw "unsupported Windows target: $Target" }
}
if (-not $HostTriplet) {
    $HostTriplet = $Triplet
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Revision = (Get-Content (Join-Path $RepoRoot 'ci/vcpkg/revision.txt') -Raw).Trim()
if ($Revision -notmatch '^[0-9a-f]{40}$') {
    throw 'Expected a full vcpkg commit SHA in ci/vcpkg/revision.txt'
}
# Use an isolated checkout; never mutate the runner's preinstalled vcpkg.
$VcpkgRoot = Join-Path $RepoRoot '.cache/vcpkg/source'
New-Item -ItemType Directory -Force -Path $VcpkgRoot | Out-Null
& git -C $VcpkgRoot init --quiet
if ($LASTEXITCODE -ne 0) { throw 'vcpkg git init failed' }
& git -C $VcpkgRoot fetch --depth 1 https://github.com/microsoft/vcpkg.git $Revision
if ($LASTEXITCODE -ne 0) { throw 'vcpkg fetch failed' }
& git -C $VcpkgRoot checkout --detach FETCH_HEAD
if ($LASTEXITCODE -ne 0) { throw 'vcpkg checkout failed' }
& (Join-Path $VcpkgRoot 'bootstrap-vcpkg.bat') -disableMetrics
if ($LASTEXITCODE -ne 0) { throw 'vcpkg bootstrap failed' }
$VcpkgExe = Join-Path $VcpkgRoot 'vcpkg.exe'
$OverlayTriplets = Join-Path $RepoRoot 'ci/vcpkg/triplets'
$env:VCPKG_OVERLAY_TRIPLETS = $OverlayTriplets
Write-Host "Pinned vcpkg revision: $Revision"
$env:VCPKG_ROOT = $VcpkgRoot
$env:VCPKG_INSTALLATION_ROOT = $VcpkgRoot
Write-Host "Using vcpkg at $VcpkgExe"

$Packages = @("glib:$Triplet", "libusb:$Triplet", "pkgconf:$Triplet")
# Native jobs use matching host tools; the manual benchmark can cross-build.
$InstallTimer = [System.Diagnostics.Stopwatch]::StartNew()
& $VcpkgExe install $Packages "--host-triplet=$HostTriplet" "--overlay-triplets=$OverlayTriplets"
if ($LASTEXITCODE -ne 0) {
  throw "vcpkg install failed with exit code $LASTEXITCODE"
}

$InstallTimer.Stop()
Write-Host ("vcpkg install elapsed: {0:N1} seconds" -f $InstallTimer.Elapsed.TotalSeconds)

& (Join-Path $PSScriptRoot 'use-windows-deps.ps1') -VcpkgRoot $VcpkgRoot -Triplet $Triplet
