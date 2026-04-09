use assert_cmd::Command;
use predicates::prelude::*;

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
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
        .stdout(predicate::str::contains("bundled resources are used by default"))
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
        .args(["devices", "list", "--library", "runtime/libdsview_runtime.so"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--library' found"));
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
