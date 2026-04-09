# Phase 1 Research

## Goal framing for this phase
- Phase 1 is no longer just docs and automation work. The repo must first become honestly portable enough to build, test, package, and validate the repository-built runtime on six targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, and `aarch64-pc-windows-msvc`.
- The public CLI contract must change from explicit runtime selection to a single repository-built runtime model. `--library` and `--use-source-runtime` should disappear from user-facing commands; `--resource-dir` remains as the explicit resource override.
- Release bundles must be self-contained enough for the default workflow to be `build -> devices list -> capture`, with bundled resources auto-discovered by stable relative paths.
- `DSView/` must remain read-only, so all portability, packaging, and workflow work has to happen in Rust crates, build scripts, native shim code under `crates/dsview-sys`, and repo automation.

## Current repo state that constrains Phase 1

### CLI surface today
- `crates/dsview-cli/src/main.rs` currently defines `devices list`, `devices open`, and `capture` as the whole supported product surface, which is a good match for a quick-start-first README.
- Every command currently flattens `SharedRuntimeArgs`, which requires `--resource-dir <PATH>` and also requires one of `--library <PATH>` or `--use-source-runtime`.
- The current runtime connect path in `connect_runtime()` rejects missing runtime selectors with `runtime_selector_missing`, so Phase 1 needs a real CLI/API refactor rather than README-only changes.
- JSON is the default output mode and text is the shell-friendly mode; this is stable enough for README examples and for artifact smoke checks that inspect machine-readable output.

### Resource behavior today
- `crates/dsview-core/src/lib.rs` validates resources through `ResourceDirectory::discover()` and `ensure_resource_file_set()`.
- The required DSLogic Plus resource set is repo-specific and already codified:
  - firmware: `DSLogicPlus.fw` or fallback `DSLogic.fw`
  - bitstreams: `DSLogicPlus.bin` and `DSLogicPlus-pgl12.bin`
- This means release packaging can be specific and minimal: the bundle only needs the exact DSLogic Plus files that `ensure_resource_file_set()` already checks, not the entire DSView resource tree.
- Today the CLI always requires a user-supplied `--resource-dir`, so Phase 1 needs an automatic discovery layer above `ResourceDirectory::discover()`.

### Runtime behavior today
- `Discovery::connect()` loads an explicit runtime library path and then applies the resource dir.
- `Discovery::connect_auto()` simply reads `dsview_sys::source_runtime_library_path()` from the build-script-injected `DSVIEW_SOURCE_RUNTIME_LIBRARY` environment and still requires an explicit resource directory argument.
- `dsview-sys` already exposes a useful separation between “load the runtime from a path” and “know whether a source runtime exists,” but the path source is compile-time and points into Cargo build output, which is not portable to packaged binaries.
- The public runtime abstraction is already narrow (`RuntimeBridge::load`, `set_firmware_resource_dir`, `init`, `list_devices`, capture methods), so Phase 1 can change runtime-location discovery without widening the unsafe boundary.

### Native build reality today
- `crates/dsview-sys/build.rs` is still Linux-centric:
  - it shells out to `cc`, `ar`, `cmake`, and `pkg-config`
  - it always emits GLib link flags from `pkg-config glib-2.0 --libs`
  - it unconditionally links `dl`
  - it hardcodes Linux header paths for the smoke shim
  - it expects the built runtime artifact at `libdsview_runtime.so`
- `crates/dsview-sys/native/CMakeLists.txt` also assumes POSIX-oriented dependencies and flags:
  - `pkg_check_modules()` for `glib-2.0`, `libusb-1.0`, and `fftw3`
  - `m` linked unconditionally
  - output naming is generic CMake shared-library naming, but Rust-side detection still assumes `.so`
- The current repo has no root `README.md` and no `.github/workflows/`, so this phase is defining both public docs and automation from scratch.

## Major conclusions that changed from the old research
- The old research assumed the honest near-term answer was Linux-first CI/release plus deferred portability work. That is no longer compatible with the clarified phase contract.
- The old research documented `--library` / `--use-source-runtime` as the accurate current UX. That is now intentionally the wrong future shape for Phase 1.
- The right planning frame is now: Phase 1 must include repo-side portability refactors first, then README/CI/release work on top of that new runtime/resource model.
- Resource packaging should not be treated as “maybe later.” The phase requires bundled resources with relative-path auto-discovery, so the runtime and README must be designed around bundle execution, not just developer `cargo run` flows.

## Repo changes needed for portable source-runtime builds

### 1. Split build-time concerns inside `dsview-sys`
The current `build.rs` mixes four different jobs that should be planned separately:
- verifying the `DSView/` submodule exists
- compiling the Rust-to-DSView dynamic bridge shim
- optionally compiling the runtime smoke shim
- configuring and building the source runtime library

Phase 1 should plan to separate these into target-aware helpers so each job can make OS/arch-specific decisions cleanly. Practical refactor seams:
- add a target descriptor derived from Cargo vars such as `CARGO_CFG_TARGET_OS`, `CARGO_CFG_TARGET_ARCH`, `CARGO_CFG_TARGET_ENV`
- compute platform-specific library naming once, then reuse it for build, runtime discovery, packaging, and tests
- emit only the link flags that actually apply to the current target
- stop keying smoke availability off Ubuntu x86_64 header paths

### 2. Make library naming target-aware instead of `.so`-only
The code path around `build_source_runtime()` and `source_runtime_library_path()` must stop assuming `libdsview_runtime.so`.

Phase 1 should plan a single helper contract such as:
- Linux: `libdsview_runtime.so`
- macOS: `libdsview_runtime.dylib`
- Windows: `dsview_runtime.dll`

That helper should drive:
- `build.rs` artifact existence checks
- bundle assembly scripts
- unpacked smoke checks
- runtime auto-discovery in `dsview-core`

### 3. Replace Linux-only tool assumptions in `build.rs`
Portable build work will require at least these changes:
- replace direct `cc`/`ar` invocation with the `cc` crate or target-aware compiler/ar selection from Cargo env so MSVC targets do not immediately fail
- gate `cargo:rustc-link-lib=dl` to only targets that need it
- make GLib include/link discovery platform-aware rather than hardcoded to `/usr/include/...` and `/usr/lib/x86_64-linux-gnu/...`
- treat the smoke shim as optional on all platforms, with availability driven by actual dependency discovery rather than path heuristics
- surface clearer build warnings/errors when required native packages are missing on each OS

### 4. Make `native/CMakeLists.txt` target-aware enough for the six-target contract
The CMake side probably needs repo-local portability work even if upstream DSView sources stay untouched.

Plan for these adjustments:
- let CMake determine the correct shared-library suffix/output and feed the resolved artifact path back to Rust via build script expectations
- make math-library linking conditional so Windows does not try to link `m`
- prefer package discovery shapes that work on macOS and Windows runners, not only Linux pkg-config layouts
- if Windows package discovery is difficult through `pkg-config`, add target-specific cache variables or environment-variable overrides in the workflow so `cmake` can find GLib/libusb/fftw/zlib without changing `DSView/`
- keep all such adaptation inside `crates/dsview-sys/native` and workflow setup, not in the submodule

### 5. Decouple non-hardware tests from source-runtime availability where useful
`crates/dsview-cli/Cargo.toml` currently pulls `dsview-sys` into CLI dev-dependencies, so even help/error-path tests inherit native build requirements.

Phase 1 does not have to remove that, but planning should note the value of reducing needless native coupling for fast matrix coverage:
- keep native-boundary tests in `dsview-sys`
- keep resource-path and artifact-path logic testable in `dsview-core`
- keep CLI parsing/help/error-shape tests runnable without hardware
- if needed, split tests so packaged-artifact validation is the only place that depends on the fully built runtime bundle

## Resource packaging and discovery strategy

### Current validated minimum resource set
Based on `ensure_resource_file_set()` in `crates/dsview-core/src/lib.rs`, the bundle only needs:
- `DSLogicPlus.fw` or `DSLogic.fw`
- `DSLogicPlus.bin`
- `DSLogicPlus-pgl12.bin`

This is an important repo-specific conclusion: Phase 1 does not need to ship the entire `DSView/DSView/res` tree if the runtime only needs these DSLogic Plus files for current scope.

### Recommended bundle layout
Use a stable layout that works the same for release artifacts and local packaged-artifact validation:

```text
<bundle-root>/
  dsview-cli[.exe]
  runtime/
    libdsview_runtime.so | libdsview_runtime.dylib | dsview_runtime.dll
  resources/
    DSLogicPlus.fw or DSLogic.fw
    DSLogicPlus.bin
    DSLogicPlus-pgl12.bin
  LICENSES/
    ...optional licensing payload...
```

Why this layout fits this repo:
- it keeps runtime and resources clearly separate
- it keeps bundle-relative discovery simple in Rust
- it lets `--resource-dir` override only the `resources/` subtree while runtime stays internal
- it avoids pointing into Cargo target directories after packaging

### Recommended runtime/resource discovery contract
`dsview-core` should own a new discovery layer that resolves both bundle defaults and explicit overrides.

Plan the contract as:
- runtime library default: path relative to the executable, e.g. `<exe_dir>/runtime/<platform-runtime-name>`
- resource directory default: path relative to the executable, e.g. `<exe_dir>/resources`
- explicit override: `--resource-dir <PATH>` replaces only the resource-dir resolution
- source-tree developer fallback for `cargo run`: if the executable-relative paths do not exist, try repo-known dev locations such as `DSView/DSView/res` and the build-script-provided source runtime path

This gives the phase one public truth while still making local development ergonomic.

### Why not infer resources from the runtime path alone
The runtime and resources have different stability requirements in this repo:
- the runtime path should become an internal implementation detail
- `--resource-dir` must remain a public override
- resource validation is already explicit and user-facing in error messages

So the clean model is: runtime discovery is internal and opaque; resource discovery is automatic by default but user-overridable.

## README content that matches the intended future UX

### Recommended top-level structure
1. Project background and value proposition
2. Quick start
3. Support posture and packaging model
4. Build requirements by platform
5. Core commands
6. Output artifacts
7. Test instructions
8. Development notes / DSView submodule constraints

### Quick start should lead with exactly this flow
The user clarified the README should be quick-start first with `build -> devices list -> capture`.

Recommended command shape after Phase 1 refactor:
- build: `cargo build --release`
- list devices: `./target/release/dsview-cli devices list`
- capture: `./target/release/dsview-cli capture --handle 1 --sample-rate-hz ... --sample-limit ... --channels ... --output out/run.vcd`
- resource override example later, not in the first happy-path example: `--resource-dir /custom/path`

### README must explain the new truth early
Immediately after quick start, document:
- only the repository-built runtime is supported
- runtime selection flags were intentionally removed
- release bundles include the runtime and DSLogic Plus resources
- default execution expects the bundled relative layout
- `--resource-dir` remains supported when users need alternate firmware/bitstream resources
- supported device scope remains `DSLogic Plus` only

### README should avoid these stale concepts
Do not carry forward old-research sections that explain:
- `--library`
- `--use-source-runtime`
- manual runtime selection
- Linux-only support posture
- bundle naming that distinguishes `cli-only` vs `bundled-runtime`

### README examples should stay narrow and truthful
Good examples for this repo and phase:
- `devices list`
- `devices open --handle <n>` only if the command still survives Phase 1 implementation
- a basic finite `capture` command that writes `.vcd`
- a `--resource-dir` override example
- `--format text` and default JSON examples only where they illustrate automation usage clearly

## Realistic GitHub Actions matrix for this repo

### One required workflow for CI
The phase contract calls for one main CI workflow that shows the whole six-target truth on pushes and pull requests.

Recommended matrix rows:
- `ubuntu-24.04`, target `x86_64-unknown-linux-gnu`
- `ubuntu-24.04-arm` or equivalent GitHub-hosted ARM runner, target `aarch64-unknown-linux-gnu`
- `macos-13`, target `x86_64-apple-darwin`
- `macos-14`, target `aarch64-apple-darwin`
- `windows-2025`, target `x86_64-pc-windows-msvc`
- `windows-11-arm` or current ARM Windows hosted runner if available, target `aarch64-pc-windows-msvc`

Important planning note: because the phase requires “all-target-or-fail,” the implementation plan must confirm runner availability first. If a hosted ARM runner is unavailable for any row, the phase will need either GitHub larger runners/self-hosted runners or a build strategy that is still honest for build+test+package validation on that target.

### Matrix job shape
Each matrix job should perform the same logical steps:
1. checkout with submodules
2. install Rust target/toolchain
3. install native prerequisites for that OS/arch
4. `cargo build --workspace --release`
5. `cargo test --workspace`
6. build the distributable bundle for that target
7. unpack the bundle into a clean temp directory
8. validate bundle layout and executable/runtime/resource discovery
9. upload the tested bundle as a workflow artifact

### Why packaged-artifact validation belongs in every matrix row
This repo’s critical Phase 1 risk is not just compilation. It is whether the final unpacked layout still lets the CLI find:
- the repository-built runtime library
- the bundled DSLogic Plus resources
- the `--resource-dir` override path when present

That is why CI should validate post-unpack execution rather than only `cargo test`.

### Candidate workflow decomposition inside one file
A single workflow file can still have clean internal structure:
- `lint` job optional, Linux-only if added
- `build-test-package` matrix job required for all six targets
- `release-readiness` aggregation job that depends on every matrix row and can be reused by the tag workflow

## Native prerequisites and workflow setup by OS

### Linux
- install `cmake`, `pkg-config`, `build-essential`, `libglib2.0-dev`, `libusb-1.0-0-dev`, `libfftw3-dev`, `zlib1g-dev`
- ensure ARM Linux runner package names match Ubuntu distro naming
- avoid x86_64-specific header assumptions in code; the workflow should not be compensating for bad hardcoded paths

### macOS
- install dependencies through Homebrew and expose paths to CMake/build.rs
- likely packages: `glib`, `libusb`, `fftw`, `pkg-config`
- validate that build.rs no longer links `dl` or assumes Linux glib include directories
- ensure the runtime output resolves as `.dylib`

### Windows
- install dependencies via a reproducible package source such as `vcpkg` or `msys2`, then pass include/lib roots into the build
- prefer MSVC targets because the phase contract names `*-pc-windows-msvc`
- replace direct `cc`/`ar` assumptions in `build.rs`
- ensure bundle smoke checks look for `dsview_runtime.dll` and can run `dsview-cli.exe`

## Artifact validation approaches without hardware

### What CI can validate honestly
Hosted CI cannot prove live capture behavior, but it can prove the portable bundle contract. For this repo, that means validating:
- archive exists and has the expected name for the target triple
- archive contains the executable, runtime library, and DSLogic Plus resource files in the expected relative layout
- unpacked CLI starts successfully enough to render help and subcommand help
- unpacked CLI can execute non-hardware code paths that depend on runtime/resource discovery logic
- `--resource-dir` override changes the resource resolution path and keeps validation logic honest

### Recommended smoke commands
Use commands that avoid requiring a connected device while still exercising bundle-relative discovery as much as possible.

Good candidates after Phase 1 runtime/resource refactor:
- `dsview-cli --help`
- `dsview-cli devices --help`
- `dsview-cli capture --help`
- a deliberately invalid capture path that fails after parsing but before hardware access, while still using default resource discovery if the command initializes discovery early enough
- a test-only or debug-only command is not recommended for this phase unless the repo already has such a pattern; it does not today

### Better repo-specific approach: add tests for discovery resolution and package them into `cargo test`
The strongest no-hardware validation for this repo is to add unit/integration coverage around:
- executable-relative runtime path resolution
- executable-relative resource path resolution
- explicit `--resource-dir` override precedence
- platform-specific runtime filename selection
- bundle layout validation helper used by packaging and CI

That lets `cargo test --workspace` validate most Phase 1 logic without requiring devices, and the post-unpack smoke check only has to prove the archive shape and executable sanity.

### Recommended artifact-validation helper
Plan a repo-local script or Rust helper that:
- receives bundle path, target triple, and expected runtime filename
- unpacks into a temp dir
- asserts required files exist
- runs the unpacked executable help commands
- optionally runs a resource-discovery validation sub-test if Phase 1 adds one

Keeping that logic in-repo avoids duplicating bundle assumptions across CI and release workflows.

## Release artifact layout and naming

### Naming recommendation
Use a single naming family across CI artifacts and releases:
- Unix archives: `dsview-cli-v{version}-{target-triple}.tar.gz`
- Windows archives: `dsview-cli-v{version}-{target-triple}.zip`

Examples:
- `dsview-cli-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `dsview-cli-v0.1.0-aarch64-unknown-linux-gnu.tar.gz`
- `dsview-cli-v0.1.0-x86_64-apple-darwin.tar.gz`
- `dsview-cli-v0.1.0-aarch64-apple-darwin.tar.gz`
- `dsview-cli-v0.1.0-x86_64-pc-windows-msvc.zip`
- `dsview-cli-v0.1.0-aarch64-pc-windows-msvc.zip`

### Internal archive root naming
Inside each archive, use a versioned directory root to avoid spilling files when unpacked:

```text
dsview-cli-v{version}-{target-triple}/
  dsview-cli[.exe]
  runtime/
  resources/
  LICENSES/
```

### Checksums and release notes
Because the phase requires checksums and packaging notes, release automation should publish:
- one checksum file covering all platform archives, e.g. `dsview-cli-v{version}-SHA256SUMS.txt`
- release notes that describe:
  - bundled runtime and resources
  - default relative-path discovery
  - `--resource-dir` override behavior
  - DSLogic Plus-only support posture

## Tag-driven release workflow recommendation

### Trigger model
- trigger on tags like `v*`
- run the same six-target build/test/package/validate matrix as CI
- only after all matrix jobs pass, create the GitHub release and upload all bundles plus checksums
- if any target fails, the workflow should stop before publishing release assets

### Recommended release workflow stages
1. matrix build/test/package/validate per target
2. artifact fan-in job downloads all validated bundles
3. checksum generation job creates a single SHA256 manifest
4. release publication job attaches all target bundles and checksum file to the tag release

### Why reuse CI packaging logic
For this repo, the biggest reliability win is to keep CI and release assembly identical. The release workflow should reuse the same package-validation helper and bundle layout logic as the PR workflow so a green PR means the tag build is exercising the same contract.

## Validation Architecture

### Validation goals
Nyquist-oriented validation for Phase 1 should prove the new portable-distribution contract, not live hardware capture. Validation should show that every promised target can:
- build the workspace and source runtime from this repo without modifying `DSView/`
- pass the workspace test suite
- produce a complete release-style bundle
- run the unpacked CLI and find bundled runtime/resources by relative path
- honor `--resource-dir` as an override while keeping default discovery stable

### Validation layers

#### 1. Unit validation
Repo tests should cover pure logic introduced by Phase 1:
- runtime filename selection by target OS
- executable-relative default runtime path resolution
- executable-relative default resource-dir resolution
- precedence rules for `--resource-dir`
- bundle layout helper expectations

Evidence source:
- `cargo test --workspace` on every matrix target

#### 2. Build validation
Each matrix job should verify the repo can build end-to-end from source:
- `cargo build --workspace --release`
- source runtime artifact exists at the target-appropriate filename
- DSView submodule presence is enforced without editing upstream code

Evidence source:
- CI logs plus package assembly step output

#### 3. Bundle validation
Each matrix job should validate the distributable artifact shape:
- archive name matches version + target triple
- archive root contains executable, `runtime/`, and `resources/`
- `resources/` includes the DSLogic Plus firmware/bitstreams checked by `ensure_resource_file_set()`
- runtime file uses the target-appropriate extension/name

Evidence source:
- package validation helper output in CI

#### 4. Post-unpack smoke validation
Each matrix job should validate the unpacked executable in a clean directory:
- executable launches successfully
- `--help` and core subcommand help work from the unpacked location
- default runtime/resource relative-path discovery does not fail immediately when using the bundle layout
- explicit `--resource-dir` override path is accepted and preferred when pointed at an alternate valid resource fixture

Evidence source:
- CI smoke command output against unpacked bundle

#### 5. Release validation
The tag workflow should only publish after all six matrix rows complete bundle validation.

Evidence source:
- release workflow dependency graph
- uploaded per-target bundles
- uploaded checksum manifest

### Validation gaps that remain after this phase
- Hosted CI still cannot validate real device discovery or capture on physical DSLogic Plus hardware.
- Hardware validation remains a separate concern from this phase’s portable packaging/distribution contract.
- README and release notes should state that CI proves build/test/package integrity across targets, not device-attachment behavior on hosted runners.

## Recommended implementation-order implications for planning
1. Refactor `dsview-sys` build/runtime portability first; otherwise README and workflows will be built around a false platform story.
2. Add runtime/resource discovery helpers in `dsview-core` so the CLI can drop public runtime-selection flags.
3. Update CLI argument surface and tests to reflect the new UX: bundled runtime by default, `--resource-dir` override only.
4. Add packaging helpers and bundle-validation helpers in-repo.
5. Author the README against the new command surface and bundle contract.
6. Add CI workflow that exercises the exact package-validation path on all six targets.
7. Add the tag-driven release workflow that reuses the same validated packaging steps and fails closed on any target failure.

## Bottom line
- This phase is now best understood as a portability-and-distribution foundation phase with README and automation as user-facing outcomes.
- The repo already has good seams for this work: `dsview-cli` for public UX, `dsview-core` for runtime/resource discovery policy, and `dsview-sys` for native portability.
- The most important repo-specific packaging insight is that the DSLogic Plus bundle can stay lean by shipping only the firmware/bitstreams that `ensure_resource_file_set()` already requires.
- The most important automation insight is that every CI/release target must validate the final unpacked bundle contract, not just `cargo build`.
