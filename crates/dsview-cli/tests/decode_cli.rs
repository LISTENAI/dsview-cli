use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

fn fixture_cli_command(fixture: &str) -> Command {
    let mut command = cli_command();
    command.env("DSVIEW_CLI_TEST_DECODE_FIXTURE", fixture);
    command
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dsview-cli-{name}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn write_config(name: &str, contents: &str) -> PathBuf {
    let dir = temp_dir(name);
    let path = dir.join("decode.json");
    fs::write(&path, contents).expect("config fixture should be written");
    path
}

fn write_input(name: &str, contents: &str) -> PathBuf {
    let dir = temp_dir(name);
    let path = dir.join("offline-input.json");
    fs::write(&path, contents).expect("offline decode input fixture should be written");
    path
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("stdout should contain valid JSON")
}

fn parse_json_file(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).expect("output artifact should be readable as bytes"),
    )
    .expect("output artifact should contain valid JSON")
}

fn fixture_config_path(path: &Path) -> &str {
    path.to_str().expect("fixture path should be valid utf-8")
}

fn valid_decode_config(name: &str) -> PathBuf {
    write_config(
        name,
        r#"{
            "version": 1,
            "decoder": {
                "id": "fixture:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            },
            "stack": [
                {
                    "id": "fixture:eeprom24xx",
                    "options": {
                        "addr_counter": 0
                    }
                }
            ]
        }"#,
    )
}

fn valid_decode_input(name: &str) -> PathBuf {
    write_input(
        name,
        r#"{
            "samplerate_hz": 1000000,
            "format": "split_logic",
            "sample_bytes": [16, 17, 18, 19],
            "unitsize": 1,
            "logic_packet_lengths": [2, 2]
        }"#,
    )
}

#[test]
fn decode_validate_accepts_valid_linear_stack_config() {
    let config = write_config(
        "decode-validate-valid",
        r#"{
            "version": 1,
            "decoder": {
                "id": "fixture:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            },
            "stack": [
                {
                    "id": "fixture:eeprom24xx",
                    "options": {
                        "addr_counter": 0
                    }
                }
            ]
        }"#,
    );

    let output = fixture_cli_command("validation-registry")
        .args(["decode", "validate", "--config", fixture_config_path(&config)])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);
    assert_eq!(json["valid"], true);
    assert_eq!(json["config_version"], 1);
    assert_eq!(json["root_decoder_id"], "fixture:i2c");
    assert_eq!(json["stack_depth"], 1);
    assert_eq!(json["bound_channel_ids"], serde_json::json!(["scl", "sda"]));

    fixture_cli_command("validation-registry")
        .args([
            "decode",
            "validate",
            "--config",
            fixture_config_path(&config),
            "--format",
            "text",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("decode config valid"))
        .stdout(predicate::str::contains("root decoder: fixture:i2c"))
        .stdout(predicate::str::contains("stack depth: 1"))
        .stdout(predicate::str::contains("bound channels: scl, sda"));
}

#[test]
fn decode_validate_rejects_missing_required_channel() {
    let config = write_config(
        "decode-validate-missing-channel",
        r#"{
            "version": 1,
            "decoder": {
                "id": "fixture:i2c",
                "channels": {
                    "scl": 0
                },
                "options": {
                    "address_format": "unshifted"
                }
            }
        }"#,
    );

    fixture_cli_command("validation-registry")
        .args(["decode", "validate", "--config", fixture_config_path(&config)])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "\"code\": \"decode_missing_required_channel\"",
        ))
        .stdout(predicate::str::contains("sda"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_validate_rejects_incompatible_stack() {
    let config = write_config(
        "decode-validate-incompatible-stack",
        r#"{
            "version": 1,
            "decoder": {
                "id": "fixture:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            },
            "stack": [
                {
                    "id": "fixture:spi-frames",
                    "options": {}
                }
            ]
        }"#,
    );

    fixture_cli_command("validation-registry")
        .args(["decode", "validate", "--config", fixture_config_path(&config)])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "\"code\": \"decode_stack_incompatible\"",
        ))
        .stdout(predicate::str::contains("fixture:spi-frames"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_validate_reports_schema_errors_with_stable_code() {
    let config = write_config(
        "decode-validate-schema-error",
        r#"{
            "version": 1,
            "decoder": {
                "id": 7,
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {}
            }
        }"#,
    );

    fixture_cli_command("validation-registry")
        .args(["decode", "validate", "--config", fixture_config_path(&config)])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "\"code\": \"decode_config_schema_invalid\"",
        ))
        .stdout(predicate::str::contains("expected a string"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_run_executes_valid_offline_decode_config() {
    let config = valid_decode_config("decode-run-valid");
    let input = valid_decode_input("decode-run-input-valid");

    let output = fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);
    assert_eq!(json["run"]["status"], "success");
    assert_eq!(json["run"]["root_decoder_id"], "fixture:i2c");
    assert_eq!(json["run"]["stack_depth"], 1);
    assert_eq!(json["run"]["sample_count"], 4);
    assert_eq!(json["run"]["event_count"], 3);
    assert_eq!(
        json["events"]
            .as_array()
            .expect("success response should include event list")
            .iter()
            .map(|event| event["decoder_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["fixture:i2c", "fixture:i2c", "fixture:eeprom24xx"]
    );
}

#[test]
fn decode_run_success_json_report_matches_contract() {
    let config = valid_decode_config("decode-run-contract-success-config");
    let input = valid_decode_input("decode-run-contract-success-input");

    let output = fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
        ])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);
    let events = json["events"]
        .as_array()
        .expect("success response should include a flat events list");

    assert_eq!(json["run"]["status"], "success");
    assert_eq!(json["run"]["root_decoder_id"], "fixture:i2c");
    assert_eq!(json["run"]["stack_depth"], 1);
    assert_eq!(json["run"]["sample_count"], 4);
    assert_eq!(json["run"]["event_count"], 3);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["decoder_id"], "fixture:i2c");
    assert_eq!(events[0]["start_sample"], 0);
    assert_eq!(events[0]["end_sample"], 2);
    assert_eq!(events[0]["annotation_class"], 0);
    assert_eq!(events[0]["annotation_type"], 0);
    assert_eq!(events[0]["texts"], serde_json::json!(["chunk-1"]));
    assert!(json.get("error").is_none());
    assert!(json.get("partial_events").is_none());
    assert!(json.get("diagnostics").is_none());
    assert!(
        !serde_json::to_string(&json)
            .expect("success contract should serialize")
            .contains("partial_success")
    );
}

#[test]
fn decode_run_failure_json_report_matches_contract() {
    let invalid_input_config = valid_decode_config("decode-run-contract-failure-config");
    let invalid_input = write_input(
        "decode-run-contract-invalid-input",
        r#"{
            "samplerate_hz": 1000000,
            "format": "split_logic",
            "sample_bytes": [1, 2, 3, 4],
            "unitsize": 1,
            "logic_packet_lengths": [2, 1]
        }"#,
    );

    let invalid_output = fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&invalid_input_config),
            "--input",
            fixture_config_path(&invalid_input),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    let invalid_json = parse_json(&invalid_output);
    assert_eq!(invalid_json["run"]["status"], "failure");
    assert_eq!(invalid_json["run"]["root_decoder_id"], "fixture:i2c");
    assert_eq!(invalid_json["run"]["stack_depth"], 1);
    assert!(invalid_json["run"].get("sample_count").is_none());
    assert!(invalid_json["run"].get("event_count").is_none());
    assert_eq!(invalid_json["error"]["code"], "decode_input_invalid");
    assert!(
        invalid_json["error"]["message"]
            .as_str()
            .expect("failure message should be present")
            .contains("logic_packet_lengths")
    );
    assert!(invalid_json.get("events").is_none());
    assert!(invalid_json.get("partial_events").is_none());
    assert!(invalid_json.get("diagnostics").is_none());

    let partial_config = valid_decode_config("decode-run-contract-partial-config");
    let partial_input = valid_decode_input("decode-run-contract-partial-input");

    let partial_output = fixture_cli_command("run-partial-failure")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&partial_config),
            "--input",
            fixture_config_path(&partial_input),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    let partial_json = parse_json(&partial_output);
    assert_eq!(partial_json["run"]["status"], "failure");
    assert_eq!(partial_json["error"]["code"], "decode_session_send_failed");
    assert!(partial_json.get("events").is_none());
    assert_eq!(
        partial_json["partial_events"][0]["decoder_id"],
        serde_json::json!("fixture:i2c")
    );
    assert_eq!(partial_json["diagnostics"]["partial_event_count"], 1);
    assert_eq!(partial_json["diagnostics"]["partial_events_available"], true);
    assert!(
        !serde_json::to_string(&partial_json)
            .expect("failure contract should serialize")
            .contains("partial_success")
    );
}

#[test]
fn decode_run_rejects_misaligned_logic_packet_lengths() {
    let config = write_config(
        "decode-run-invalid-input-config",
        r#"{
            "version": 1,
            "decoder": {
                "id": "fixture:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            }
        }"#,
    );
    let input = write_input(
        "decode-run-invalid-input",
        r#"{
            "samplerate_hz": 1000000,
            "format": "split_logic",
            "sample_bytes": [1, 2, 3, 4],
            "unitsize": 1,
            "logic_packet_lengths": [2, 1]
        }"#,
    );

    fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"decode_input_invalid\""))
        .stdout(predicate::str::contains("logic_packet_lengths"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_run_text_output_is_summary_focused() {
    let config = valid_decode_config("decode-run-text-summary-config");
    let input = valid_decode_input("decode-run-text-summary-input");

    fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
            "--format",
            "text",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("decode run succeeded"))
        .stdout(predicate::str::contains("root decoder: fixture:i2c"))
        .stdout(predicate::str::contains("stack depth: 1"))
        .stdout(predicate::str::contains("sample count: 4"))
        .stdout(predicate::str::contains("event count: 3"))
        .stdout(predicate::str::contains("chunk-1").not())
        .stdout(predicate::str::contains("fixture-complete").not())
        .stdout(predicate::str::contains("fixture:eeprom24xx").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_run_fails_when_runtime_execution_errors() {
    let config = valid_decode_config("decode-run-runtime-error-config");
    let input = valid_decode_input("decode-run-runtime-error-input");

    let output = fixture_cli_command("run-runtime-failure")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);
    assert_eq!(json["run"]["status"], "failure");
    assert_eq!(json["error"]["code"], "decode_session_send_failed");
    assert_eq!(json["diagnostics"]["partial_event_count"], 0);
    assert!(json.get("partial_events").is_none());
}

#[test]
fn decode_run_success_writes_output_artifact() {
    let config = valid_decode_config("decode-run-output-success-config");
    let input = valid_decode_input("decode-run-output-success-input");
    let output_dir = temp_dir("decode-run-output-success");
    let output_path = output_dir.join("decode-report.json");

    fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
            "--output",
            fixture_config_path(&output_path),
        ])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let json = parse_json_file(&output_path);
    assert_eq!(json["run"]["status"], "success");
    assert_eq!(json["run"]["event_count"], 3);
    assert_eq!(
        json["events"].as_array().expect("output artifact should contain events").len(),
        3
    );
}

#[test]
fn decode_run_failure_writes_failure_report_when_output_requested() {
    let config = valid_decode_config("decode-run-output-failure-config");
    let input = valid_decode_input("decode-run-output-failure-input");
    let output_dir = temp_dir("decode-run-output-failure");
    let output_path = output_dir.join("decode-report.json");

    let stdout = fixture_cli_command("run-partial-failure")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
            "--output",
            fixture_config_path(&output_path),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    let stdout_json = parse_json(&stdout);
    let file_json = parse_json_file(&output_path);

    assert_eq!(stdout_json["run"]["status"], "failure");
    assert_eq!(stdout_json["error"]["code"], "decode_session_send_failed");
    assert_eq!(stdout_json["diagnostics"]["partial_event_count"], 1);
    assert_eq!(
        stdout_json["partial_events"][0]["decoder_id"],
        serde_json::json!("fixture:i2c")
    );
    assert_eq!(file_json, stdout_json);
}

#[test]
fn decode_run_stdout_and_file_output_share_the_same_schema() {
    let config = valid_decode_config("decode-run-output-schema-config");
    let input = valid_decode_input("decode-run-output-schema-input");
    let output_dir = temp_dir("decode-run-output-schema");
    let output_path = output_dir.join("decode-report.json");

    let stdout = fixture_cli_command("run-success")
        .args([
            "decode",
            "run",
            "--config",
            fixture_config_path(&config),
            "--input",
            fixture_config_path(&input),
            "--output",
            fixture_config_path(&output_path),
        ])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    assert_eq!(parse_json(&stdout), parse_json_file(&output_path));
}
