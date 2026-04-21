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

fn fixture_config_path(path: &Path) -> &str {
    path.to_str().expect("fixture path should be valid utf-8")
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
    let config = write_config(
        "decode-run-valid",
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
    let input = write_input(
        "decode-run-input-valid",
        r#"{
            "samplerate_hz": 1000000,
            "format": "split_logic",
            "sample_bytes": [16, 17, 18, 19],
            "unitsize": 1,
            "logic_packet_lengths": [2, 2]
        }"#,
    );

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
    assert_eq!(json["ok"], true);
    assert_eq!(json["root_decoder_id"], "fixture:i2c");
    assert_eq!(json["stack_depth"], 1);
    assert_eq!(json["sample_count"], 4);
    assert_eq!(json["annotation_count"], 3);
    assert_eq!(
        json["annotation_decoder_ids"],
        serde_json::json!(["fixture:eeprom24xx", "fixture:i2c"])
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
fn decode_run_fails_when_runtime_execution_errors() {
    let config = write_config(
        "decode-run-runtime-error-config",
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
        "decode-run-runtime-error-input",
        r#"{
            "samplerate_hz": 1000000,
            "format": "split_logic",
            "sample_bytes": [170, 187, 204, 221],
            "unitsize": 1
        }"#,
    );

    fixture_cli_command("run-runtime-failure")
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
        .stdout(predicate::str::contains("\"code\": \"decode_run_failed\""))
        .stdout(predicate::str::contains("send logic chunk"))
        .stderr(predicate::str::is_empty());
}
