# Windows CI dependencies

`revision.txt` pins the vcpkg ports and bootstrap metadata to a full upstream
commit. `scripts/setup-windows-deps.ps1` checks it out under `.cache/vcpkg/source`
without modifying the runner's preinstalled vcpkg.

The native x64 and ARM64 jobs use the matching `*-windows-release` triplet for
both target libraries and host tools. These triplets retain dynamic libraries
and the dynamic CRT, but build only Release. Windows tests run with
`cargo test --workspace --profile ci-test`. This Release-derived profile keeps
`debug_assertions` enabled for CLI test fixtures and writes binaries to a
separate `ci-test` directory. Cargo reports `PROFILE=release` to build scripts,
so native runtimes still use Release dependencies. The packaged executable
always comes from the normal Release profile, with fixture hooks disabled.
Unix jobs retain Debug test coverage.
Changing Windows tests back to Debug requires restoring Debug dependencies too.

The binary/download cache is separate from the old dual-configuration cache.
Its key includes the target, MSVC tools version, revision, triplets and setup
script. vcpkg additionally checks package ABI hashes before reusing binaries.
The cache is saved immediately after dependency installation succeeds, so a
later compile/test/package failure does not discard the dependency build.

## Measuring changes

Compare cold runs on the same runner image, compiler and pinned revision. A
genuine cold run must have empty installed, binary and download directories;
check that the logs report zero restored packages. Do not delete shared caches
to force a measurement: use a separate cache namespace or disable restoration
in an experimental workflow instead. Record checkout/bootstrap overhead as well
as the per-port and total install durations printed by the setup step.

The historical dual-configuration reference is run `42946935635`: dependency
setup took 25m49s on x64 and 73m22s on ARM64. It used the runner's unpinned vcpkg,
so comparisons against it are indicative, not controlled benchmarks.

Next experiments should isolate gettext host tools or cross-build dependencies
on x64, retaining native ARM64 build, tests and bundle validation. Do not remove
GLib's gettext-libintl/libiconv dependencies or replace tools with unchecked
downloads merely to shorten the benchmark.

The CI workflow has a manual `benchmark-arm64-deps` boolean input. When enabled,
it builds dependencies on `windows-2022`, using ARM64 target
libraries and x64 host tools. It disables binary caches, uses a fresh download
directory, and uploads vcpkg build logs even on failure. A following native
ARM64 job consumes a raw vcpkg export and runs the project build, optimized
tests and bundle validation. Its bundles are experiment artifacts; it does not
replace the normal native ARM64 CI job. For example:

```powershell
gh workflow run ci.yml -R LISTENAI/dsview-cli --ref codex/feature-windows-cold-build -f benchmark-arm64-deps=true
```

An ordinary manual dispatch (input false) runs the full build/test/package
matrix. Cross-build timing alone is not sufficient to switch production CI;
the exported dependencies must also pass native ARM64 tests and bundle checks.

FFTW is not installed or linked: only the upstream GUI spectrum implementation
uses it, and that source is not part of either CLI native runtime.
