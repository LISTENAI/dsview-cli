# Windows CI dependencies

`revision.txt` pins the vcpkg ports and bootstrap metadata to a full upstream
commit. `scripts/setup-windows-deps.ps1` checks it out under `.cache/vcpkg/source`
without modifying the runner's preinstalled vcpkg.

## Build layout

Windows x64 builds its dependencies on its native runner. Windows ARM64 uses
`windows-arm64.yml`, shared by CI and Release:

1. A Windows x64 runner cross-builds ARM64 target libraries and x64 host tools.
2. vcpkg exports the installed dependencies as a raw archive.
3. A native Windows ARM64 runner downloads the archive, builds the CLI, runs
   tests and validates the bundle. Production binaries and tests run on ARM64.

Both target and host triplets build only Release, retaining dynamic libraries
and the dynamic CRT. `scripts/use-windows-deps.ps1` activates either a normal
vcpkg installation or the exported dependencies using the same environment.
The dependency artifact is retained for one day. Release publication downloads
only `dsview-cli-*` artifacts, excluding the intermediate dependency archive.

Windows tests run with `cargo test --workspace --profile ci-test`. This
Release-derived profile keeps `debug_assertions` enabled for CLI test fixtures
and writes binaries to a separate `ci-test` directory. Cargo reports
`PROFILE=release` to build scripts, so native runtimes use Release dependencies.
The packaged executable comes from the ordinary Release profile with fixture
hooks disabled. Unix jobs retain Debug test coverage. Changing Windows tests
back to Debug requires restoring Debug dependencies too.

FFTW is not installed or linked: only the upstream GUI spectrum implementation
uses it, and that source is not part of either CLI native runtime.

## Caching and measurement

Native x64 and cross-built ARM64 dependencies use separate binary/download
cache namespaces. Keys include the target/host configuration, runner image,
MSVC tools version, revision, triplets and setup script. Image changes get a new
cache key, avoiding an immutable cache hit that cannot save rebuilt binaries. vcpkg additionally checks package
ABI hashes before reusing binaries. Caches are saved immediately after a
successful dependency installation, so later build/test failures do not discard
the dependency build.

For a cold ARM64 benchmark, manually run CI with both `benchmark-arm64-deps`
and `cold-build` enabled. This runs only the shared ARM64 pipeline and disables
cache restore/save and vcpkg binary caching:

```powershell
gh workflow run ci.yml -R LISTENAI/dsview-cli --ref codex/feature-windows-cold-build -f benchmark-arm64-deps=true -f cold-build=true
```

An ordinary manual dispatch runs the full build/test/package matrix. Use the
same runner image, compiler and pinned revision for controlled comparisons.
Record checkout/bootstrap, export/transfer and native validation overhead as
well as the per-port and total install durations. Do not delete shared caches
to force a measurement.

The historical dual-configuration reference is run `42946935635`: dependency
setup took 25m49s on x64 and 73m22s on ARM64. It used the runner's unpinned vcpkg,
so comparisons against it are indicative, not controlled benchmarks. The first
Release-only x64 validation in run `42981133683` took 16m02s for dependency setup
and passed all 285 tests and bundle validation. The isolated cross-build in run
`42981349885` took 1064.3 seconds for vcpkg install with binary caching disabled.

Run `42983849027` validated the complete cold cross-build path: dependency job
18m25s, native ARM64 build/test/package job 4m28s, elapsed chain 22m55s. All
285 native ARM64 tests and bundle validation passed. This is about 70% less
elapsed time than the historical 77m35s native ARM64 job.
