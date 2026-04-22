use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

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

fn fixture_cli_command(fixture: &str) -> Command {
    let mut command = cli_command();
    command.env("DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE", fixture);
    command
}

fn mock_cli_command() -> Command {
    fixture_cli_command("1")
}

fn unique_capture_paths(stem: &str) -> (PathBuf, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dsview-cli-capture-{stem}-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temporary capture dir should be created");
    (dir.join("capture.vcd"), dir.join("capture.json"))
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("stdout should contain valid JSON")
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
        .stdout(predicate::str::contains("stop-option:").not())
        .stdout(predicate::str::contains("buffer-100x16").not());
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
            "--channel-mode",
            "stream-100x16",
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

#[test]
fn capture_json_success_reports_requested_and_effective_device_options() {
    let (vcd_path, metadata_path) = unique_capture_paths("json-success");
    let output = mock_cli_command()
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "4097",
            "--channels",
            "0,1,2,3,4,5,6,7",
            "--operation-mode",
            "buffer",
            "--stop-option",
            "stop-after-samples",
            "--channel-mode",
            "buffer-200x8",
            "--threshold-volts",
            "2.4",
            "--filter",
            "off",
            "--output",
            vcd_path.to_str().unwrap(),
            "--metadata-output",
            metadata_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let payload = parse_json(&output);
    assert_eq!(payload["completion"], "clean_success");
    assert_eq!(payload["artifacts"]["vcd_path"], vcd_path.display().to_string());
    assert_eq!(
        payload["artifacts"]["metadata_path"],
        metadata_path.display().to_string()
    );
    assert_eq!(
        payload["device_options"]["requested"]["operation_mode_id"],
        "operation-mode:0"
    );
    assert_eq!(
        payload["device_options"]["requested"]["sample_limit"],
        4097
    );
    assert_eq!(
        payload["device_options"]["effective"]["channel_mode_id"],
        "channel-mode:21"
    );
    assert_eq!(
        payload["device_options"]["effective"]["sample_limit"],
        5120
    );

    let metadata = parse_json(&fs::read(metadata_path).expect("metadata should be written"));
    assert_eq!(metadata["schema_version"], 2);
    assert_eq!(
        metadata["device_options"]["requested"]["sample_limit"],
        4097
    );
    assert_eq!(
        metadata["device_options"]["effective"]["sample_limit"],
        5120
    );
}

#[test]
fn capture_text_success_reports_effective_device_options_concisely() {
    let (vcd_path, metadata_path) = unique_capture_paths("text-success");
    let output = mock_cli_command()
        .args([
            "capture",
            "--format",
            "text",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "4097",
            "--channels",
            "0,1,2,3,4,5,6,7",
            "--operation-mode",
            "buffer",
            "--stop-option",
            "stop-after-samples",
            "--channel-mode",
            "buffer-200x8",
            "--threshold-volts",
            "2.4",
            "--filter",
            "off",
            "--output",
            vcd_path.to_str().unwrap(),
            "--metadata-output",
            metadata_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).expect("text output should be utf-8");
    assert!(text.contains("capture clean_success"));
    assert!(text.contains("effective options:"));
    assert!(text.contains("operation mode: operation-mode:0"));
    assert!(text.contains("stop option: stop-option:1"));
    assert!(text.contains("channel mode: channel-mode:21"));
    assert!(text.contains("enabled channels: 0,1,2,3,4,5,6,7"));
    assert!(text.contains("threshold volts: 2.4"));
    assert!(text.contains("filter: filter:1"));
    assert!(text.contains("sample rate: 100000000"));
    assert!(text.contains("sample limit: 5120"));
    assert!(text.contains(&format!("vcd {}", vcd_path.display())));
    assert!(text.contains(&format!("metadata {}", metadata_path.display())));
    assert!(!text.contains("requested options:"));
    assert!(!text.contains("4097"));
}

#[test]
fn capture_apply_failure_reports_applied_steps_and_failed_step() {
    let (vcd_path, metadata_path) = unique_capture_paths("apply-failure");
    let output = fixture_cli_command("apply-failure-filter")
        .args([
            "capture",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "4097",
            "--channels",
            "0,1,2,3,4,5,6,7",
            "--operation-mode",
            "buffer",
            "--stop-option",
            "stop-after-samples",
            "--channel-mode",
            "buffer-200x8",
            "--threshold-volts",
            "2.4",
            "--filter",
            "off",
            "--output",
            vcd_path.to_str().unwrap(),
            "--metadata-output",
            metadata_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let payload = parse_json(&output);
    assert_eq!(payload["code"], "device_option_apply_failed");
    assert_eq!(payload["native_error"], "SR_ERR_ARG");
    let detail = payload["detail"]
        .as_str()
        .expect("apply failure should include detail");
    assert!(detail.contains("applied_steps=operation_mode,stop_option,channel_mode,threshold_volts"));
    assert!(detail.contains("failed_step=filter"));
}

#[test]
fn capture_without_overrides_reports_inherited_effective_device_options() {
    let (vcd_path, metadata_path) = unique_capture_paths("baseline");
    let output = mock_cli_command()
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
            "--output",
            vcd_path.to_str().unwrap(),
            "--metadata-output",
            metadata_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let payload = parse_json(&output);
    assert_eq!(payload["completion"], "clean_success");
    assert_eq!(payload["artifacts"]["vcd_path"], vcd_path.display().to_string());
    assert_eq!(
        payload["artifacts"]["metadata_path"],
        metadata_path.display().to_string()
    );
    assert_eq!(
        payload["device_options"]["requested"],
        payload["device_options"]["effective"]
    );
    assert_eq!(
        payload["device_options"]["effective"]["operation_mode_id"],
        "operation-mode:0"
    );
    assert_eq!(
        payload["device_options"]["effective"]["channel_mode_id"],
        "channel-mode:20"
    );
    assert_eq!(
        payload["device_options"]["effective"]["enabled_channels"],
        serde_json::json!([0, 1, 2, 3])
    );

    let metadata = parse_json(&fs::read(metadata_path).expect("metadata should be written"));
    assert_eq!(metadata["schema_version"], 2);
    assert_eq!(
        metadata["device_options"]["requested"],
        metadata["device_options"]["effective"]
    );
}
