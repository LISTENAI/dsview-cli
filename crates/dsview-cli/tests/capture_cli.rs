use assert_cmd::Command;
use predicates::prelude::*;

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

fn mock_cli_command() -> Command {
    let mut command = cli_command();
    command.env("DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE", "1");
    command
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
fn capture_help_lists_device_option_flags_and_points_to_devices_options() {
    cli_command()
        .arg("capture")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--operation-mode <TOKEN>"))
        .stdout(predicate::str::contains("--stop-option <TOKEN>"))
        .stdout(predicate::str::contains("--channel-mode <TOKEN>"))
        .stdout(predicate::str::contains("--threshold-volts <VOLTS>"))
        .stdout(predicate::str::contains("--filter <TOKEN>"))
        .stdout(predicate::str::contains("--channels <IDX[,IDX...]>"))
        .stdout(predicate::str::contains("Comma-separated logic channel indexes to enable"))
        .stdout(predicate::str::contains("devices options --handle <HANDLE>"))
        .stdout(predicate::str::contains("possible values").not());
}

#[test]
fn capture_accepts_all_device_option_flags_before_runtime_validation() {
    mock_cli_command()
        .args([
            "capture",
            "--format",
            "text",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--operation-mode",
            "buffer",
            "--stop-option",
            "stop-after-samples",
            "--channel-mode",
            "buffer-100x16",
            "--threshold-volts",
            "1.8",
            "--filter",
            "off",
            "--output",
            "artifacts/run.bin",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("capture_output_path_invalid:"))
        .stderr(predicate::str::contains("must use the .vcd extension"))
        .stderr(predicate::str::contains("unexpected argument").not())
        .stderr(predicate::str::contains("invalid value").not());
}

#[test]
fn capture_rejects_unknown_operation_mode_token() {
    mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--operation-mode",
            "turbo-buffer",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"operation_mode_unsupported\""))
        .stdout(predicate::str::contains("turbo-buffer"))
        .stdout(predicate::str::contains("devices options --handle <HANDLE>"))
        .stdout(predicate::str::contains("possible values").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_rejects_unknown_stop_option_token() {
    mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--operation-mode",
            "buffer",
            "--stop-option",
            "never-stop",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"stop_option_unsupported\""))
        .stdout(predicate::str::contains("never-stop"))
        .stdout(predicate::str::contains("possible values").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_rejects_unknown_channel_mode_token() {
    mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--operation-mode",
            "buffer",
            "--channel-mode",
            "buffer-400x2",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"channel_mode_unsupported\""))
        .stdout(predicate::str::contains("buffer-400x2"))
        .stdout(predicate::str::contains("possible values").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_rejects_unknown_filter_token() {
    mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--filter",
            "smoothing-9000",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"filter_unsupported\""))
        .stdout(predicate::str::contains("smoothing-9000"))
        .stdout(predicate::str::contains("possible values").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_rejects_invalid_threshold_volts_value() {
    cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--threshold-volts",
            "not-a-number",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("invalid value 'not-a-number'"))
        .stderr(predicate::str::contains("--threshold-volts <VOLTS>"));
}

#[test]
fn capture_reports_stable_validation_error_for_incompatible_device_option_combination() {
    mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--operation-mode",
            "stream",
            "--stop-option",
            "stop-after-samples",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"stop_option_incompatible\""))
        .stdout(predicate::str::contains("stop-option:1"))
        .stdout(predicate::str::contains("operation-mode:1"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_reports_stable_validation_error_for_channel_limit_exceeded() {
    mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3,4,5,6,7,8",
            "--operation-mode",
            "buffer",
            "--channel-mode",
            "buffer-200x8",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "\"code\": \"enabled_channels_exceed_mode_limit\"",
        ))
        .stdout(predicate::str::contains("9"))
        .stdout(predicate::str::contains("8"))
        .stderr(predicate::str::is_empty());
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
