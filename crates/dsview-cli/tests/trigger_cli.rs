//! Capture trigger CLI contract tests.
//!
//! These run the debug-only capture fixture (handle 7 with
//! `DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE`), so they exercise argument
//! parsing, trigger validation, and output shape without hardware.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

fn fixture_cli_command(fixture: &str) -> Command {
    let mut command = cli_command();
    command.env("DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE", fixture);
    command
}

fn unique_dir(stem: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "dsview-cli-trigger-{stem}-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temporary dir should be created");
    dir
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("stdout should contain valid JSON")
}

/// Buffer-mode fixture capture of 4096 samples on channels 0-3.
fn buffer_capture_args(dir: &Path) -> Vec<String> {
    [
        "capture",
        "--handle",
        "7",
        "--sample-rate-hz",
        "100000000",
        "--sample-limit",
        "4096",
        "--channels",
        "0,1,2,3",
        "--operation-mode",
        "buffer",
        "--channel-mode",
        "buffer-100x16",
        "--output",
    ]
    .into_iter()
    .map(str::to_string)
    .chain([dir.join("capture.vcd").display().to_string()])
    .collect()
}

fn stream_capture_args(dir: &Path) -> Vec<String> {
    [
        "capture",
        "--handle",
        "7",
        "--sample-rate-hz",
        "100000000",
        "--sample-limit",
        "4096",
        "--channels",
        "0,1",
        "--operation-mode",
        "stream",
        "--channel-mode",
        "stream-100x16",
        "--output",
    ]
    .into_iter()
    .map(str::to_string)
    .chain([dir.join("capture.vcd").display().to_string()])
    .collect()
}

fn failure_code(output: &[u8]) -> String {
    parse_json(output)["code"]
        .as_str()
        .expect("error JSON should carry a code")
        .to_string()
}

fn run_failure(args: &[String]) -> Value {
    let output = fixture_cli_command("1")
        .args(args)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    parse_json(&output)
}

#[test]
fn capture_help_lists_trigger_flags() {
    cli_command()
        .args(["capture", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Trigger:"))
        .stdout(predicate::str::contains("--trigger <CH=COND[,CH=COND...]>"))
        .stdout(predicate::str::contains("low, high, rising, falling, edge"))
        .stdout(predicate::str::contains("--trigger-config <PATH>"))
        .stdout(predicate::str::contains("--trigger-position <PERCENT>"));
}

#[test]
fn simple_trigger_capture_reports_requested_and_observed_trigger() {
    let dir = unique_dir("simple");
    let mut args = buffer_capture_args(&dir);
    args.extend(
        ["--trigger", "0=rising,D3=high", "--trigger-position", "25"]
            .into_iter()
            .map(str::to_string),
    );

    let output = fixture_cli_command("1")
        .args(&args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let payload = parse_json(&output);
    let expected = json!({
        "requested": {
            "mode": "simple",
            "channels": {"0": "rising", "3": "high"},
            "position_percent": 25
        },
        "observed": {
            "trigger_packet": true,
            "fired": true,
            "sample_index": 1024,
            "raw_status": 1
        }
    });
    assert_eq!(payload["completion"], "clean_success");
    assert_eq!(payload["trigger"], expected);

    let metadata = parse_json(&fs::read(dir.join("capture.json")).expect("metadata is written"));
    assert_eq!(metadata["schema_version"], 2);
    assert_eq!(metadata["trigger"], expected);
}

#[test]
fn simple_trigger_text_output_reports_the_trigger_sample() {
    let dir = unique_dir("text");
    let mut args = buffer_capture_args(&dir);
    args.extend(
        ["--trigger", "1=falling", "--format", "text"]
            .into_iter()
            .map(str::to_string),
    );

    fixture_cli_command("1")
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "trigger simple fired at sample 40 (position 1%)",
        ));
}

#[test]
fn capture_without_trigger_keeps_the_previous_output_shape() {
    let dir = unique_dir("none");
    let output = fixture_cli_command("1")
        .args(buffer_capture_args(&dir))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(parse_json(&output).get("trigger").is_none());

    let metadata = parse_json(&fs::read(dir.join("capture.json")).expect("metadata is written"));
    assert!(metadata.get("trigger").is_none());
}

#[test]
fn stage_trigger_config_file_is_reported_in_metadata() {
    let dir = unique_dir("stage");
    let config = dir.join("trigger.json");
    fs::write(
        &config,
        r#"{
            "version": 1,
            "mode": "stage",
            "stages": [
                {"pattern_a": {"channels": {"0": "rising"}}},
                {"pattern_a": {"channels": {"1": "high", "2": "low"}}, "count": 3, "contiguous": true}
            ]
        }"#,
    )
    .unwrap();
    let mut args = buffer_capture_args(&dir);
    args.extend([
        "--trigger-config".to_string(),
        config.display().to_string(),
        "--trigger-position".to_string(),
        "50".to_string(),
    ]);

    let output = fixture_cli_command("1")
        .args(&args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let requested = &parse_json(&output)["trigger"]["requested"];
    assert_eq!(requested["mode"], "stage");
    assert_eq!(requested["position_percent"], 50);
    assert_eq!(requested["stages"].as_array().unwrap().len(), 2);
    assert_eq!(
        requested["stages"][1],
        json!({
            "pattern_a": {"channels": {"1": "high", "2": "low"}, "invert": false},
            "pattern_b": {"channels": {}, "invert": false},
            "combine": "and",
            "count": 3,
            "contiguous": true
        })
    );
}

#[test]
fn serial_trigger_config_normalizes_the_data_value() {
    let dir = unique_dir("serial");
    let config = dir.join("trigger.json");
    fs::write(
        &config,
        r#"{"mode":"serial","start":{"2":"rising"},"stop":{"2":"falling"},"clock":{"0":"rising"},"data_channel":1,"data_bits":8,"data_value":"0x5A"}"#,
    )
    .unwrap();
    let mut args = buffer_capture_args(&dir);
    args.extend(["--trigger-config".to_string(), config.display().to_string()]);

    let output = fixture_cli_command("1")
        .args(&args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let requested = &parse_json(&output)["trigger"]["requested"];
    assert_eq!(requested["mode"], "serial");
    assert_eq!(requested["data_channel"], 1);
    assert_eq!(requested["data_value"], "01011010");
}

#[test]
fn serial_trigger_without_edge_flags_is_rejected() {
    let dir = unique_dir("serial-levels");
    let config = dir.join("trigger.json");
    fs::write(
        &config,
        r#"{"mode":"serial","start":{"2":"high"},"stop":{"2":"low"},"clock":{"0":"rising"},"data_channel":1,"data_bits":4,"data_value":"0101"}"#,
    )
    .unwrap();
    let mut args = buffer_capture_args(&dir);
    args.extend(["--trigger-config".to_string(), config.display().to_string()]);

    let error = run_failure(&args);
    assert_eq!(error["code"], "trigger_config_invalid");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("start needs at least one edge")
    );
}

#[test]
fn trigger_on_a_disabled_channel_is_rejected() {
    let dir = unique_dir("disabled");
    let mut args = buffer_capture_args(&dir);
    args.extend(["--trigger".to_string(), "5=high".to_string()]);

    let error = run_failure(&args);
    assert_eq!(error["code"], "trigger_channel_not_enabled");
    assert!(error["message"].as_str().unwrap().contains("--channels"));
}

#[test]
fn trigger_position_outside_the_buffer_range_is_rejected() {
    let dir = unique_dir("position");
    let mut args = buffer_capture_args(&dir);
    args.extend(
        ["--trigger", "0=high", "--trigger-position", "95"]
            .into_iter()
            .map(str::to_string),
    );

    assert_eq!(run_failure(&args)["code"], "trigger_position_out_of_range");
}

#[test]
fn stream_mode_accepts_simple_trigger_but_rejects_stage_trigger_and_position() {
    let dir = unique_dir("stream");

    let mut simple = stream_capture_args(&dir);
    simple.extend(["--trigger".to_string(), "0=rising".to_string()]);
    let output = fixture_cli_command("1")
        .args(&simple)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        parse_json(&output)["trigger"]["requested"]["position_percent"],
        1
    );

    let mut positioned = stream_capture_args(&dir);
    positioned.extend(
        ["--trigger", "0=rising", "--trigger-position", "10"]
            .into_iter()
            .map(str::to_string),
    );
    assert_eq!(
        run_failure(&positioned)["code"],
        "trigger_position_out_of_range"
    );

    let config = dir.join("stage.json");
    fs::write(
        &config,
        r#"{"mode":"stage","stages":[{"pattern_a":{"channels":{"0":"rising"}}}]}"#,
    )
    .unwrap();
    let mut stage = stream_capture_args(&dir);
    stage.extend(["--trigger-config".to_string(), config.display().to_string()]);
    assert_eq!(run_failure(&stage)["code"], "trigger_mode_unsupported");
}

#[test]
fn malformed_trigger_requests_fail_before_runtime_work() {
    let dir = unique_dir("malformed");

    for trigger in ["0=up", "rising", "x=high", "0=high,0=low"] {
        let mut args = buffer_capture_args(&dir);
        args.extend(["--trigger".to_string(), trigger.to_string()]);
        // No fixture env: parsing must fail before any runtime or device work.
        let output = cli_command()
            .args(&args)
            .assert()
            .failure()
            .code(1)
            .get_output()
            .stdout
            .clone();
        assert_eq!(failure_code(&output), "trigger_config_invalid", "{trigger}");
    }

    let config = dir.join("bad.json");
    fs::write(
        &config,
        r#"{"mode":"simple","channels":{"0":"rising"},"bogus":true}"#,
    )
    .unwrap();
    let mut args = buffer_capture_args(&dir);
    args.extend(["--trigger-config".to_string(), config.display().to_string()]);
    let output = cli_command()
        .args(&args)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    assert_eq!(failure_code(&output), "trigger_config_invalid");

    let mut args = buffer_capture_args(&dir);
    args.extend([
        "--trigger-config".to_string(),
        dir.join("missing.json").display().to_string(),
    ]);
    let output = cli_command()
        .args(&args)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    assert_eq!(failure_code(&output), "trigger_config_unreadable");
}

#[test]
fn trigger_flags_conflict_and_position_requires_a_trigger() {
    let dir = unique_dir("conflict");

    let mut both = buffer_capture_args(&dir);
    both.extend(
        ["--trigger", "0=high", "--trigger-config", "trigger.json"]
            .into_iter()
            .map(str::to_string),
    );
    cli_command()
        .args(&both)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));

    let mut position_only = buffer_capture_args(&dir);
    position_only.extend(["--trigger-position".to_string(), "10".to_string()]);
    cli_command()
        .args(&position_only)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ));
}

#[test]
fn trigger_that_never_fires_reports_a_dedicated_error_code() {
    let dir = unique_dir("not-fired");
    let mut args = buffer_capture_args(&dir);
    args.extend(["--trigger".to_string(), "0=rising".to_string()]);

    let output = fixture_cli_command("trigger-not-fired")
        .args(&args)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let error = parse_json(&output);
    assert_eq!(error["code"], "capture_trigger_not_fired");
    let detail = error["detail"].as_str().unwrap();
    assert!(detail.starts_with("capture_timeout"), "{detail}");
    assert!(detail.contains("--wait-timeout-ms"), "{detail}");
    assert_eq!(error["cleanup"]["stop_succeeded"], true);
}
