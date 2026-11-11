param(
    [Parameter(Mandatory = $true)]
    [string]$VcpkgRoot,
    [Parameter(Mandatory = $true)]
    [ValidateSet('x64-windows-release', 'arm64-windows-release')]
    [string]$Triplet
)

$ErrorActionPreference = 'Stop'
$VcpkgRoot = (Resolve-Path -LiteralPath $VcpkgRoot).Path
$OverlayTriplets = Join-Path (Split-Path -Parent $PSScriptRoot) 'ci/vcpkg/triplets'
foreach ($RequiredFile in @('scripts/buildsystems/vcpkg.cmake', "installed/$Triplet/tools/pkgconf/pkgconf.exe")) {
    if (-not (Test-Path -LiteralPath (Join-Path $VcpkgRoot $RequiredFile))) {
        throw "Missing Windows dependency file: $RequiredFile"
    }
}

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
