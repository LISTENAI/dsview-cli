#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PATCH_DIR="${REPO_ROOT}/patches/dsview"
DSVIEW_DIR="${REPO_ROOT}/DSView"

if [[ ! -d "${DSVIEW_DIR}" ]]; then
  echo "DSView submodule directory not found: ${DSVIEW_DIR}" >&2
  exit 1
fi

if [[ ! -d "${PATCH_DIR}" ]]; then
  echo "No DSView patch directory found: ${PATCH_DIR}" >&2
  exit 1
fi

shopt -s nullglob
patches=("${PATCH_DIR}"/*.patch)
shopt -u nullglob

if [[ ${#patches[@]} -eq 0 ]]; then
  echo "No DSView patches to apply in ${PATCH_DIR}"
  exit 0
fi

for patch in "${patches[@]}"; do
  name="$(basename "${patch}")"
  apply_args=(--ignore-space-change --ignore-whitespace)

  if git -C "${DSVIEW_DIR}" apply "${apply_args[@]}" --check "${patch}" >/dev/null 2>&1; then
    echo "Applying ${name}"
    git -C "${DSVIEW_DIR}" apply "${apply_args[@]}" "${patch}"
    continue
  fi

  if git -C "${DSVIEW_DIR}" apply "${apply_args[@]}" --reverse --check "${patch}" >/dev/null 2>&1; then
    echo "Already applied ${name}"
    continue
  fi

  echo "Patch ${name} does not apply cleanly. Inspect DSView working tree before proceeding." >&2
  exit 1
done

echo "DSView patch application complete."
