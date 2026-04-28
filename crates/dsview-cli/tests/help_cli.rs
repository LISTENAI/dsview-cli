use assert_cmd::Command;
use predicates::prelude::*;

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

fn assert_help_contains(args: &[&str], expected: &str) {
    cli_command()
        .args(args)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn top_level_help_describes_major_commands() {
    assert_help_contains(&[], "Discover and inspect connected DSLogic devices");
    assert_help_contains(
        &[],
        "List, inspect, validate, and run offline protocol decoders",
    );
    assert_help_contains(
        &[],
        "Capture logic samples from a selected device into VCD artifacts",
    );
}

#[test]
fn devices_help_describes_device_subcommands() {
    assert_help_contains(&["devices"], "List selectable devices and their handles");
    assert_help_contains(&["devices"], "Open a device handle to verify it is usable");
    assert_help_contains(
        &["devices"],
        "Show supported capture options and current values for a device",
    );
}

#[test]
fn decode_help_describes_decode_subcommands() {
    assert_help_contains(&["decode"], "List available protocol decoders");
    assert_help_contains(
        &["decode"],
        "Inspect decoder channels, options, and metadata",
    );
    assert_help_contains(
        &["decode"],
        "Validate a decode config without running a session",
    );
    assert_help_contains(
        &["decode"],
        "Run an offline decode session and emit a report",
    );
}

#[test]
fn leaf_command_help_repeats_command_descriptions() {
    let cases: &[(&[&str], &str)] = &[
        (
            &["devices", "list"],
            "List selectable devices and their handles",
        ),
        (
            &["devices", "open"],
            "Open a device handle to verify it is usable",
        ),
        (
            &["devices", "options"],
            "Show supported capture options and current values for a device",
        ),
        (&["decode", "list"], "List available protocol decoders"),
        (
            &["decode", "inspect"],
            "Inspect decoder channels, options, and metadata",
        ),
        (
            &["decode", "validate"],
            "Validate a decode config without running a session",
        ),
        (
            &["decode", "run"],
            "Run an offline decode session and emit a report",
        ),
        (
            &["capture"],
            "Capture logic samples from a selected device into VCD artifacts",
        ),
    ];

    for (args, expected) in cases {
        assert_help_contains(args, expected);
    }
}
