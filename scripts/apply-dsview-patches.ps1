Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$patchDir = Join-Path $repoRoot "patches\dsview"
$dsviewDir = Join-Path $repoRoot "DSView"
$applyArgs = @("--ignore-space-change", "--ignore-whitespace")

if (-not (Test-Path -LiteralPath $dsviewDir -PathType Container)) {
    throw "DSView submodule directory not found: $dsviewDir"
}

if (-not (Test-Path -LiteralPath $patchDir -PathType Container)) {
    throw "No DSView patch directory found: $patchDir"
}

$patches = @(Get-ChildItem -LiteralPath $patchDir -Filter *.patch -File | Sort-Object Name)
if ($patches.Count -eq 0) {
    Write-Host "No DSView patches to apply in $patchDir"
    exit 0
}

foreach ($patch in $patches) {
    $name = $patch.Name
    $patchPath = $patch.FullName

    $null = & git -C $dsviewDir apply @applyArgs --check $patchPath 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Applying $name"
        & git -C $dsviewDir apply @applyArgs $patchPath
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to apply $name"
        }
        continue
    }

    $null = & git -C $dsviewDir apply @applyArgs --reverse --check $patchPath 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Already applied $name"
        continue
    }

    throw "Patch $name does not apply cleanly. Inspect DSView working tree before proceeding."
}

Write-Host "DSView patch application complete."
