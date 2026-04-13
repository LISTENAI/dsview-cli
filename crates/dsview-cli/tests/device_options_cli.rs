use assert_cmd::Command;
use dsview_cli::{build_device_options_response, render_device_options_text};
use dsview_core::{
    ChannelModeGroupSnapshot, ChannelModeOptionSnapshot, CurrentDeviceOptionValues,
    DeviceIdentitySnapshot, DeviceOptionsSnapshot, EnumOptionSnapshot,
    LegacyThresholdMetadataSnapshot, RawOptionMetadataSnapshot, ThresholdCapabilitySnapshot,
};
use predicates::prelude::*;
use serde_json::json;

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

fn sample_snapshot() -> DeviceOptionsSnapshot {
    DeviceOptionsSnapshot {
        device: DeviceIdentitySnapshot {
            selection_handle: 7,
            native_handle: 42,
            stable_id: "dslogic-plus".to_string(),
            kind: "DSLogic Plus".to_string(),
            name: "DSLogic PLus".to_string(),
        },
        current: CurrentDeviceOptionValues {
            operation_mode_id: Some("operation-mode:0".to_string()),
            operation_mode_code: Some(0),
            stop_option_id: Some("stop-option:1".to_string()),
            stop_option_code: Some(1),
            filter_id: Some("filter:1".to_string()),
            filter_code: Some(1),
            channel_mode_id: Some("channel-mode:20".to_string()),
            channel_mode_code: Some(20),
        },
        operation_modes: vec![
            EnumOptionSnapshot {
                id: "operation-mode:0".to_string(),
                native_code: 0,
                label: "Buffer Mode".to_string(),
            },
            EnumOptionSnapshot {
                id: "operation-mode:1".to_string(),
                native_code: 1,
                label: "Stream Mode".to_string(),
            },
        ],
        stop_options: vec![
            EnumOptionSnapshot {
                id: "stop-option:0".to_string(),
                native_code: 0,
                label: "Immediate".to_string(),
            },
            EnumOptionSnapshot {
                id: "stop-option:1".to_string(),
                native_code: 1,
                label: "Stop after samples".to_string(),
            },
        ],
        filters: vec![
            EnumOptionSnapshot {
                id: "filter:1".to_string(),
                native_code: 1,
                label: "Off".to_string(),
            },
            EnumOptionSnapshot {
                id: "filter:2".to_string(),
                native_code: 2,
                label: "1 Sample".to_string(),
            },
        ],
        channel_modes_by_operation_mode: vec![
            ChannelModeGroupSnapshot {
                operation_mode_id: "operation-mode:0".to_string(),
                operation_mode_code: 0,
                current_channel_mode_id: Some("channel-mode:20".to_string()),
                current_channel_mode_code: Some(20),
                channel_modes: vec![
                    ChannelModeOptionSnapshot {
                        id: "channel-mode:20".to_string(),
                        native_code: 20,
                        label: "Buffer 100x16".to_string(),
                        max_enabled_channels: 16,
                    },
                    ChannelModeOptionSnapshot {
                        id: "channel-mode:21".to_string(),
                        native_code: 21,
                        label: "Buffer 200x8".to_string(),
                        max_enabled_channels: 8,
                    },
                ],
            },
            ChannelModeGroupSnapshot {
                operation_mode_id: "operation-mode:1".to_string(),
                operation_mode_code: 1,
                current_channel_mode_id: None,
                current_channel_mode_code: None,
                channel_modes: vec![ChannelModeOptionSnapshot {
                    id: "channel-mode:30".to_string(),
                    native_code: 30,
                    label: "Stream 100x16".to_string(),
                    max_enabled_channels: 16,
                }],
            },
        ],
        threshold: ThresholdCapabilitySnapshot {
            id: "threshold:vth-range".to_string(),
            kind: "voltage-range".to_string(),
            current_volts: Some(1.8),
            min_volts: 0.7,
            max_volts: 5.0,
            step_volts: 0.1,
            legacy_metadata: Some(LegacyThresholdMetadataSnapshot {
                current_native_code: Some(3),
                options: vec![
                    RawOptionMetadataSnapshot {
                        native_code: 3,
                        label: "1.8 V".to_string(),
                    },
                    RawOptionMetadataSnapshot {
                        native_code: 5,
                        label: "3.3 V".to_string(),
                    },
                ],
            }),
        },
    }
}

struct CaptureAcceptanceTokens {
    operation_mode: &'static str,
    stop_option: &'static str,
    channel_mode: &'static str,
    threshold_volts: f64,
    filter: &'static str,
}

fn capture_acceptance_tokens() -> CaptureAcceptanceTokens {
    CaptureAcceptanceTokens {
        operation_mode: "buffer",
        stop_option: "stop-after-samples",
        channel_mode: "buffer-100x16",
        threshold_volts: 1.8,
        filter: "off",
    }
}

fn capture_flag_examples() -> [&'static str; 6] {
    [
        "  --operation-mode buffer",
        "  --stop-option stop-after-samples",
        "  --channel-mode buffer-100x16",
        "  --threshold-volts 1.8",
        "  --filter off",
        "  --channels IDX[,IDX...]",
    ]
}

#[test]
fn device_options_json_exposes_capture_tokens_and_stable_ids() {
    let response = build_device_options_response(&sample_snapshot());
    let actual = serde_json::to_value(&response).expect("response should serialize");

    let expected = json!({
        "selected_handle": 7,
        "device": {
            "stable_id": "dslogic-plus",
            "model": "DSLogic Plus",
            "native_name": "DSLogic PLus",
            "native_handle": 42
        },
        "current": {
            "operation_mode_token": "buffer",
            "operation_mode_stable_id": "operation-mode:0",
            "operation_mode_code": 0,
            "stop_option_token": "stop-after-samples",
            "stop_option_stable_id": "stop-option:1",
            "stop_option_code": 1,
            "filter_token": "off",
            "filter_stable_id": "filter:1",
            "filter_code": 1,
            "channel_mode_token": "buffer-100x16",
            "channel_mode_stable_id": "channel-mode:20",
            "channel_mode_code": 20
        },
        "operation_modes": [
            {
                "token": "buffer",
                "stable_id": "operation-mode:0",
                "label": "Buffer Mode",
                "native_code": 0
            },
            {
                "token": "stream",
                "stable_id": "operation-mode:1",
                "label": "Stream Mode",
                "native_code": 1
            }
        ],
        "stop_options": [
            {
                "token": "immediate",
                "stable_id": "stop-option:0",
                "label": "Immediate",
                "native_code": 0
            },
            {
                "token": "stop-after-samples",
                "stable_id": "stop-option:1",
                "label": "Stop after samples",
                "native_code": 1
            }
        ],
        "filters": [
            {
                "token": "off",
                "stable_id": "filter:1",
                "label": "Off",
                "native_code": 1
            },
            {
                "token": "1-sample",
                "stable_id": "filter:2",
                "label": "1 Sample",
                "native_code": 2
            }
        ],
        "threshold": {
            "threshold_flag": "--threshold-volts",
            "stable_id": "threshold:vth-range",
            "kind": "voltage-range",
            "current_volts": 1.8,
            "min_volts": 0.7,
            "max_volts": 5.0,
            "step_volts": 0.1,
            "legacy_metadata": {
                "current_native_code": 3,
                "options": [
                    {
                        "native_code": 3,
                        "label": "1.8 V"
                    },
                    {
                        "native_code": 5,
                        "label": "3.3 V"
                    }
                ]
            }
        },
        "capture_guide": {
            "operation_mode_flag": "--operation-mode",
            "stop_option_flag": "--stop-option",
            "channel_mode_flag": "--channel-mode",
            "threshold_flag": "--threshold-volts",
            "filter_flag": "--filter",
            "channels_flag": "--channels"
        }
    });

    assert_eq!(actual["selected_handle"], expected["selected_handle"]);
    assert_eq!(actual["device"], expected["device"]);
    assert_eq!(actual["current"], expected["current"]);
    assert_eq!(actual["operation_modes"], expected["operation_modes"]);
    assert_eq!(actual["stop_options"], expected["stop_options"]);
    assert_eq!(actual["filters"], expected["filters"]);
    assert_eq!(actual["threshold"], expected["threshold"]);
    assert_eq!(actual["capture_guide"], expected["capture_guide"]);
}

#[test]
fn device_options_json_retains_channel_mode_limits_for_channels_flag() {
    let response = build_device_options_response(&sample_snapshot());
    let actual = serde_json::to_value(&response).expect("response should serialize");

    let expected = json!({
        "channels_flag": "--channels",
        "channel_modes_by_operation_mode": [
            {
                "operation_mode_token": "buffer",
                "operation_mode_stable_id": "operation-mode:0",
                "operation_mode_code": 0,
                "current_channel_mode_token": "buffer-100x16",
                "current_channel_mode_stable_id": "channel-mode:20",
                "current_channel_mode_code": 20,
                "channel_modes": [
                    {
                        "token": "buffer-100x16",
                        "stable_id": "channel-mode:20",
                        "label": "Buffer 100x16",
                        "native_code": 20,
                        "max_enabled_channels": 16
                    },
                    {
                        "token": "buffer-200x8",
                        "stable_id": "channel-mode:21",
                        "label": "Buffer 200x8",
                        "native_code": 21,
                        "max_enabled_channels": 8
                    }
                ]
            },
            {
                "operation_mode_token": "stream",
                "operation_mode_stable_id": "operation-mode:1",
                "operation_mode_code": 1,
                "current_channel_mode_token": null,
                "current_channel_mode_stable_id": null,
                "current_channel_mode_code": null,
                "channel_modes": [
                    {
                        "token": "stream-100x16",
                        "stable_id": "channel-mode:30",
                        "label": "Stream 100x16",
                        "native_code": 30,
                        "max_enabled_channels": 16
                    }
                ]
            }
        ]
    });

    assert_eq!(actual["capture_guide"]["channels_flag"], expected["channels_flag"]);
    assert_eq!(
        actual["channel_modes_by_operation_mode"],
        expected["channel_modes_by_operation_mode"]
    );
}

#[test]
fn device_options_text_shows_copy_paste_capture_flags() {
    let response = build_device_options_response(&sample_snapshot());
    let rendered = render_device_options_text(&response);

    let expected = concat!(
        "device\n",
        "  selected_handle: 7\n",
        "  stable_id: dslogic-plus\n",
        "  model: DSLogic Plus\n",
        "  native_name: DSLogic PLus\n",
        "  native_handle: 42\n",
        "capture_flags\n",
        "  --operation-mode buffer\n",
        "  --stop-option stop-after-samples\n",
        "  --channel-mode buffer-100x16\n",
        "  --threshold-volts 1.8\n",
        "  --filter off\n",
        "  --channels IDX[,IDX...]  (buffer-100x16 allows up to 16 enabled channels)\n",
        "operation_modes\n",
        "  current: buffer | stable_id=operation-mode:0 | native_code=0\n",
        "  option: token=buffer | stable_id=operation-mode:0 | native_code=0 | label=Buffer Mode\n",
        "  option: token=stream | stable_id=operation-mode:1 | native_code=1 | label=Stream Mode\n",
        "stop_options\n",
        "  current: stop-after-samples | stable_id=stop-option:1 | native_code=1\n",
        "  option: token=immediate | stable_id=stop-option:0 | native_code=0 | label=Immediate\n",
        "  option: token=stop-after-samples | stable_id=stop-option:1 | native_code=1 | label=Stop after samples\n",
        "channel_modes_by_operation_mode\n",
        "  group: buffer | stable_id=operation-mode:0 | native_code=0 | current_channel_mode=buffer-100x16 | current_stable_id=channel-mode:20 | current_native_code=20\n",
        "  channel_mode: buffer-100x16 | stable_id=channel-mode:20 | native_code=20 | label=Buffer 100x16 | max_enabled_channels=16\n",
        "  channel_mode: buffer-200x8 | stable_id=channel-mode:21 | native_code=21 | label=Buffer 200x8 | max_enabled_channels=8\n",
        "  group: stream | stable_id=operation-mode:1 | native_code=1 | current_channel_mode=none | current_stable_id=none | current_native_code=none\n",
        "  channel_mode: stream-100x16 | stable_id=channel-mode:30 | native_code=30 | label=Stream 100x16 | max_enabled_channels=16\n",
        "threshold\n",
        "  --threshold-volts 1.8\n",
        "  stable_id: threshold:vth-range\n",
        "  kind: voltage-range\n",
        "  min_volts: 0.7\n",
        "  max_volts: 5.0\n",
        "  step_volts: 0.1\n",
        "  legacy_current_native_code: 3\n",
        "  legacy_option: native_code=3 | label=1.8 V\n",
        "  legacy_option: native_code=5 | label=3.3 V\n",
        "filters\n",
        "  current: off | stable_id=filter:1 | native_code=1\n",
        "  option: token=off | stable_id=filter:1 | native_code=1 | label=Off\n",
        "  option: token=1-sample | stable_id=filter:2 | native_code=2 | label=1 Sample"
    );

    assert_eq!(rendered, expected);
}

#[test]
fn device_options_text_lists_capture_flag_examples_in_flag_order() {
    let response = build_device_options_response(&sample_snapshot());
    let rendered = render_device_options_text(&response);
    let expected_lines = capture_flag_examples();

    let mut previous_index = None;
    for expected_line in expected_lines {
        let index = rendered
            .find(expected_line)
            .unwrap_or_else(|| panic!("expected line `{expected_line}` in rendered text"));
        if let Some(previous_index) = previous_index {
            assert!(
                index > previous_index,
                "expected `{expected_line}` after the previous capture flag example"
            );
        }
        previous_index = Some(index);
    }
}

#[test]
fn device_options_json_uses_same_tokens_as_capture_flags() {
    let response = build_device_options_response(&sample_snapshot());
    let actual = serde_json::to_value(&response).expect("response should serialize");
    let tokens = capture_acceptance_tokens();
    let operation_mode_tokens: Vec<_> = actual["operation_modes"]
        .as_array()
        .expect("operation modes should be an array")
        .iter()
        .map(|option| option["token"].as_str().expect("token should be a string"))
        .collect();
    let stop_option_tokens: Vec<_> = actual["stop_options"]
        .as_array()
        .expect("stop options should be an array")
        .iter()
        .map(|option| option["token"].as_str().expect("token should be a string"))
        .collect();
    let filter_tokens: Vec<_> = actual["filters"]
        .as_array()
        .expect("filters should be an array")
        .iter()
        .map(|option| option["token"].as_str().expect("token should be a string"))
        .collect();
    let channel_mode_tokens: Vec<_> = actual["channel_modes_by_operation_mode"]
        .as_array()
        .expect("channel mode groups should be an array")
        .iter()
        .flat_map(|group| {
            group["channel_modes"]
                .as_array()
                .expect("channel modes should be an array")
                .iter()
                .map(|mode| mode["token"].as_str().expect("token should be a string"))
                .collect::<Vec<_>>()
        })
        .collect();

    assert_eq!(actual["current"]["operation_mode_token"], tokens.operation_mode);
    assert_eq!(actual["current"]["stop_option_token"], tokens.stop_option);
    assert_eq!(actual["current"]["channel_mode_token"], tokens.channel_mode);
    assert_eq!(actual["threshold"]["current_volts"], json!(tokens.threshold_volts));
    assert_eq!(actual["current"]["filter_token"], tokens.filter);
    assert!(operation_mode_tokens.contains(&tokens.operation_mode));
    assert!(stop_option_tokens.contains(&tokens.stop_option));
    assert!(filter_tokens.contains(&tokens.filter));
    assert!(channel_mode_tokens.contains(&tokens.channel_mode));
    assert_eq!(
        actual["capture_guide"]["operation_mode_flag"],
        "--operation-mode"
    );
    assert_eq!(actual["capture_guide"]["stop_option_flag"], "--stop-option");
    assert_eq!(actual["capture_guide"]["channel_mode_flag"], "--channel-mode");
    assert_eq!(actual["capture_guide"]["threshold_flag"], "--threshold-volts");
    assert_eq!(actual["capture_guide"]["filter_flag"], "--filter");
    assert_eq!(actual["capture_guide"]["channels_flag"], "--channels");
}

#[test]
fn devices_options_help_mentions_handle_and_output_contract() {
    cli_command()
        .args(["devices", "options", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--handle <HANDLE>"))
        .stdout(predicate::str::contains("Selection handle returned by `devices list`"))
        .stdout(predicate::str::contains("json is stable for automation"))
        .stdout(predicate::str::contains("text is for direct shell use"));
}

#[test]
fn devices_options_invalid_handle_fails_before_runtime_in_json_mode() {
    cli_command()
        .args(["devices", "options", "--handle", "0"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"invalid_selector\""))
        .stdout(predicate::str::contains(
            "--handle must be a non-zero device handle from `devices list`.",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn devices_root_help_lists_options_subcommand() {
    cli_command()
        .args(["devices", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("options"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("open"));
}
