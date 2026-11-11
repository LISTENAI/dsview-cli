param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')]
    [string]$Target
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
# Host tools also use the Release-only triplet on these native runners.
$InstallTimer = [System.Diagnostics.Stopwatch]::StartNew()
& $VcpkgExe install $Packages "--host-triplet=$Triplet" "--overlay-triplets=$OverlayTriplets"
if ($LASTEXITCODE -ne 0) {
  throw "vcpkg install failed with exit code $LASTEXITCODE"
}

$InstallTimer.Stop()
Write-Host ("vcpkg install elapsed: {0:N1} seconds" -f $InstallTimer.Elapsed.TotalSeconds)

$PkgConfigExe = "$VcpkgRoot/installed/$Triplet/tools/pkgconf/pkgconf.exe"
$PkgConfigPath = "$VcpkgRoot/installed/$Triplet/lib/pkgconfig;$VcpkgRoot/installed/$Triplet/share/pkgconfig"
$PkgConfigBin = "$VcpkgRoot/installed/$Triplet/tools/pkgconf"
$RuntimeBin = "$VcpkgRoot/installed/$Triplet/bin"

"VCPKG_ROOT=$VcpkgRoot" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"VCPKG_INSTALLATION_ROOT=$VcpkgRoot" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"VCPKG_OVERLAY_TRIPLETS=$OverlayTriplets" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"DSVIEW_VCPKG_ROOT=$VcpkgRoot" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"DSVIEW_VCPKG_TRIPLET=$Triplet" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"CMAKE_TOOLCHAIN_FILE=$VcpkgRoot/scripts/buildsystems/vcpkg.cmake" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"PKG_CONFIG=$PkgConfigExe" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"PKG_CONFIG_PATH=$PkgConfigPath" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"PKG_CONFIG_LIBDIR=$PkgConfigPath" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"PATH=$RuntimeBin;$PkgConfigBin;$env:PATH" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
