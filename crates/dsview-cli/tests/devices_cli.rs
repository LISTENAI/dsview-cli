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

fn expected_build_version() -> &'static str {
    match option_env!("DSVIEW_BUILD_VERSION") {
        Some(version) => version,
        None => env!("CARGO_PKG_VERSION"),
    }
}

#[test]
fn version_flag_reports_the_resolved_build_version() {
    cli_command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_build_version()));
}

#[test]
fn devices_help_does_not_expose_runtime_selection_flags() {
    cli_command()
        .arg("devices")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("open"))
        .stdout(predicate::str::contains("--library").not())
        .stdout(predicate::str::contains("--use-source-runtime").not());
}

#[test]
fn devices_list_help_keeps_resource_override_only() {
    cli_command()
        .args(["devices", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--resource-dir <PATH>"))
        .stdout(predicate::str::contains(
            "bundled resources are used by default",
        ))
        .stdout(predicate::str::contains("--library").not())
        .stdout(predicate::str::contains("--use-source-runtime").not());
}

#[test]
fn devices_open_help_keeps_resource_override_only() {
    cli_command()
        .args(["devices", "open", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--resource-dir <PATH>"))
        .stdout(predicate::str::contains("--handle <HANDLE>"))
        .stdout(predicate::str::contains("--library").not())
        .stdout(predicate::str::contains("--use-source-runtime").not());
}

#[test]
fn devices_list_rejects_removed_library_flag() {
    cli_command()
        .args([
            "devices",
            "list",
            "--library",
            "runtime/libdsview_runtime.so",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--library' found",
        ));
}

#[test]
fn devices_open_rejects_removed_use_source_runtime_flag() {
    cli_command()
        .args(["devices", "open", "--use-source-runtime", "--handle", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--use-source-runtime' found",
        ));
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("stdout should contain valid JSON")
}

#[test]
fn decode_list_json_reports_canonical_decoder_ids() {
    let output = fixture_cli_command("registry")
        .args(["decode", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);
    assert_eq!(json["decoders"][0]["id"], "0:i2c");
    assert_eq!(json["decoders"][0]["required_channel_ids"][0], "scl");
    assert_eq!(json["decoders"][0]["optional_channel_ids"][0], "sda");
}

#[test]
fn decode_inspect_json_includes_channels_options_and_stack_metadata() {
    let output = fixture_cli_command("registry")
        .args(["decode", "inspect", "0:i2c"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);
    assert_eq!(json["decoder"]["id"], "0:i2c");
    assert_eq!(json["decoder"]["required_channels"][0]["id"], "scl");
    assert_eq!(json["decoder"]["optional_channels"][0]["id"], "sda");
    assert_eq!(json["decoder"]["options"][0]["id"], "address_format");
    assert_eq!(json["decoder"]["annotation_rows"][0]["id"], "frames");
    assert_eq!(json["decoder"]["inputs"][0]["id"], "logic");
    assert_eq!(json["decoder"]["outputs"][1]["id"], "i2c-messages");
}

#[test]
fn decode_list_text_is_human_readable_but_keeps_canonical_ids_visible() {
    fixture_cli_command("registry")
        .args(["decode", "list", "--format", "text"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0:i2c"))
        .stdout(predicate::str::contains("Inter-Integrated Circuit"))
        .stdout(predicate::str::contains("required: scl"))
        .stdout(predicate::str::contains("outputs: i2c, i2c-messages"));
}

#[test]
fn decode_inspect_reports_unknown_decoder_cleanly() {
    fixture_cli_command("registry")
        .args(["decode", "inspect", "missing-decoder"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"decoder_not_found\""))
        .stdout(predicate::str::contains("missing-decoder"))
        .stdout(predicate::str::contains("decode list"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_list_reports_missing_runtime_cleanly() {
    fixture_cli_command("missing-runtime")
        .args(["decode", "list"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"decode_runtime_missing\""))
        .stdout(predicate::str::contains("decoder runtime"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn decode_list_reports_missing_decoder_metadata_cleanly() {
    fixture_cli_command("missing-metadata")
        .args(["decode", "list"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"decoder_metadata_missing\""))
        .stdout(predicate::str::contains("decoder metadata"))
        .stderr(predicate::str::is_empty());
}
