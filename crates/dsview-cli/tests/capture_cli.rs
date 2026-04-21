use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

fn ensure_placeholder_runtime() {
    let exe = assert_cmd::cargo::cargo_bin("dsview-cli");
    let runtime_dir = exe
        .parent()
        .expect("CLI test binary should live in a directory")
        .join("runtime");
    let runtime_name = if cfg!(target_os = "windows") {
        "dsview_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libdsview_runtime.dylib"
    } else {
        "libdsview_runtime.so"
    };

    fs::create_dir_all(&runtime_dir).expect("runtime directory should be creatable for CLI tests");
    fs::write(runtime_dir.join(runtime_name), b"placeholder")
        .expect("placeholder runtime should be writable for CLI tests");
}

#[test]
fn capture_help_mentions_bundled_resource_override_and_artifact_controls() {
    cli_command()
        .arg("capture")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--resource-dir <PATH>"))
        .stdout(predicate::str::contains("bundled resources are used by default"))
        .stdout(predicate::str::contains("--handle <HANDLE>"))
        .stdout(predicate::str::contains("Selection handle returned by `devices list`"))
        .stdout(predicate::str::contains("--sample-rate-hz <HZ>"))
        .stdout(predicate::str::contains("Requested capture sample rate in hertz"))
        .stdout(predicate::str::contains("--sample-limit <SAMPLES>"))
        .stdout(predicate::str::contains("Requested sample count before the finite capture stops"))
        .stdout(predicate::str::contains("--channels <IDX[,IDX...]>"))
        .stdout(predicate::str::contains("Comma-separated logic channel indexes to enable"))
        .stdout(predicate::str::contains("--output <PATH>"))
        .stdout(predicate::str::contains("--metadata-output <PATH>"))
        .stdout(predicate::str::contains("--wait-timeout-ms <WAIT_TIMEOUT_MS>"))
        .stdout(predicate::str::contains("Maximum time to wait for capture completion before aborting"))
        .stdout(predicate::str::contains("--poll-interval-ms <POLL_INTERVAL_MS>"))
        .stdout(predicate::str::contains("Polling interval for checking capture progress while waiting"))
        .stdout(predicate::str::contains("json is stable for automation"))
        .stdout(predicate::str::contains("must end with .vcd"))
        .stdout(predicate::str::contains("defaults to the VCD path with a .json extension"))
        .stdout(predicate::str::contains("--library").not())
        .stdout(predicate::str::contains("--use-source-runtime").not());
}

#[test]
fn capture_rejects_removed_runtime_selection_flags() {
    cli_command()
        .args(["capture", "--library", "runtime/libdsview_runtime.so"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--library' found"));

    cli_command()
        .args(["capture", "--use-source-runtime"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--use-source-runtime' found",
        ));
}

#[test]
fn capture_invalid_output_path_fails_on_stderr_in_text_mode() {
    cli_command()
        .args([
            "capture",
            "--format",
            "text",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.bin",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("capture_output_path_invalid:"))
        .stderr(predicate::str::contains("must use the .vcd extension"));
}

#[test]
fn capture_invalid_metadata_output_fails_on_stderr_in_text_mode() {
    cli_command()
        .args([
            "capture",
            "--format",
            "text",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.vcd",
            "--metadata-output",
            "artifacts/run.txt",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("capture_metadata_output_path_invalid:"))
        .stderr(predicate::str::contains("must use the .json extension"));
}

#[test]
fn capture_conflicting_artifact_paths_fail_before_runtime_work() {
    cli_command()
        .args([
            "capture",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.vcd",
            "--metadata-output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"capture_artifact_paths_conflict\""))
        .stdout(predicate::str::contains("must be different"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_missing_resource_files_reports_bundle_relative_guidance() {
    ensure_placeholder_runtime();

    cli_command()
        .args([
            "capture",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"resource_files_missing\""))
        .stdout(predicate::str::contains("required DSLogic Plus files"))
        .stdout(predicate::str::contains("resources/"))
        .stderr(predicate::str::is_empty());
}
