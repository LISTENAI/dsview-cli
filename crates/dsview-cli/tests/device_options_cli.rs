use dsview_cli::{build_device_options_response, render_device_options_text};
use dsview_core::{
    ChannelModeGroupSnapshot, ChannelModeOptionSnapshot, CurrentDeviceOptionValues,
    DeviceIdentitySnapshot, DeviceOptionsSnapshot, EnumOptionSnapshot,
    LegacyThresholdMetadataSnapshot, RawOptionMetadataSnapshot, ThresholdCapabilitySnapshot,
};
use serde_json::json;

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
            filter_id: Some("filter:2".to_string()),
            filter_code: Some(2),
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

#[test]
fn device_options_json_is_stable_for_automation() {
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
            "operation_mode_id": "operation-mode:0",
            "operation_mode_code": 0,
            "stop_option_id": "stop-option:1",
            "stop_option_code": 1,
            "filter_id": "filter:2",
            "filter_code": 2,
            "channel_mode_id": "channel-mode:20",
            "channel_mode_code": 20
        },
        "operation_modes": [
            {
                "id": "operation-mode:0",
                "label": "Buffer Mode",
                "native_code": 0
            },
            {
                "id": "operation-mode:1",
                "label": "Stream Mode",
                "native_code": 1
            }
        ],
        "stop_options": [
            {
                "id": "stop-option:0",
                "label": "Immediate",
                "native_code": 0
            },
            {
                "id": "stop-option:1",
                "label": "Stop after samples",
                "native_code": 1
            }
        ],
        "filters": [
            {
                "id": "filter:1",
                "label": "Off",
                "native_code": 1
            },
            {
                "id": "filter:2",
                "label": "1 Sample",
                "native_code": 2
            }
        ],
        "threshold": {
            "id": "threshold:vth-range",
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
        "channel_modes_by_operation_mode": [
            {
                "operation_mode_id": "operation-mode:0",
                "operation_mode_code": 0,
                "current_channel_mode_id": "channel-mode:20",
                "current_channel_mode_code": 20,
                "channel_modes": [
                    {
                        "id": "channel-mode:20",
                        "label": "Buffer 100x16",
                        "native_code": 20,
                        "max_enabled_channels": 16
                    },
                    {
                        "id": "channel-mode:21",
                        "label": "Buffer 200x8",
                        "native_code": 21,
                        "max_enabled_channels": 8
                    }
                ]
            },
            {
                "operation_mode_id": "operation-mode:1",
                "operation_mode_code": 1,
                "current_channel_mode_id": null,
                "current_channel_mode_code": null,
                "channel_modes": [
                    {
                        "id": "channel-mode:30",
                        "label": "Stream 100x16",
                        "native_code": 30,
                        "max_enabled_channels": 16
                    }
                ]
            }
        ]
    });

    assert_eq!(actual, expected);
}

#[test]
fn device_options_text_uses_deterministic_section_order() {
    let response = build_device_options_response(&sample_snapshot());
    let rendered = render_device_options_text(&response);

    let expected = concat!(
        "device\n",
        "  selected_handle: 7\n",
        "  stable_id: dslogic-plus\n",
        "  model: DSLogic Plus\n",
        "  native_name: DSLogic PLus\n",
        "  native_handle: 42\n",
        "operation_modes\n",
        "  current: operation-mode:0 | native_code=0\n",
        "  option: operation-mode:0 | native_code=0 | label=Buffer Mode\n",
        "  option: operation-mode:1 | native_code=1 | label=Stream Mode\n",
        "stop_options\n",
        "  current: stop-option:1 | native_code=1\n",
        "  option: stop-option:0 | native_code=0 | label=Immediate\n",
        "  option: stop-option:1 | native_code=1 | label=Stop after samples\n",
        "filters\n",
        "  current: filter:2 | native_code=2\n",
        "  option: filter:1 | native_code=1 | label=Off\n",
        "  option: filter:2 | native_code=2 | label=1 Sample\n",
        "threshold\n",
        "  id: threshold:vth-range\n",
        "  kind: voltage-range\n",
        "  current_volts: 1.8\n",
        "  min_volts: 0.7\n",
        "  max_volts: 5.0\n",
        "  step_volts: 0.1\n",
        "  legacy_current_native_code: 3\n",
        "  legacy_option: native_code=3 | label=1.8 V\n",
        "  legacy_option: native_code=5 | label=3.3 V\n",
        "channel_modes_by_operation_mode\n",
        "  group: operation-mode:0 | native_code=0 | current_channel_mode=channel-mode:20 | current_native_code=20\n",
        "  channel_mode: channel-mode:20 | native_code=20 | label=Buffer 100x16 | max_enabled_channels=16\n",
        "  channel_mode: channel-mode:21 | native_code=21 | label=Buffer 200x8 | max_enabled_channels=8\n",
        "  group: operation-mode:1 | native_code=1 | current_channel_mode=none | current_native_code=none\n",
        "  channel_mode: channel-mode:30 | native_code=30 | label=Stream 100x16 | max_enabled_channels=16"
    );

    assert_eq!(rendered, expected);
}
