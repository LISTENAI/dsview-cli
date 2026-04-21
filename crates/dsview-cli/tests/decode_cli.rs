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
