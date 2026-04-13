use dsview_core::{
    ChannelModeGroupSnapshot, CurrentDeviceOptionValues, DeviceOptionsSnapshot, EnumOptionSnapshot,
    ThresholdCapabilitySnapshot,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceIdentityResponse {
    pub stable_id: String,
    pub model: String,
    pub native_name: String,
    pub native_handle: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceOptionsResponse {
    pub selected_handle: u64,
    pub device: DeviceIdentityResponse,
    pub current: CurrentDeviceOptionValues,
    pub operation_modes: Vec<EnumOptionSnapshot>,
    pub stop_options: Vec<EnumOptionSnapshot>,
    pub filters: Vec<EnumOptionSnapshot>,
    pub threshold: ThresholdCapabilitySnapshot,
    pub channel_modes_by_operation_mode: Vec<ChannelModeGroupSnapshot>,
}

pub fn build_device_options_response(snapshot: &DeviceOptionsSnapshot) -> DeviceOptionsResponse {
    DeviceOptionsResponse {
        selected_handle: snapshot.device.selection_handle,
        device: DeviceIdentityResponse {
            stable_id: snapshot.device.stable_id.clone(),
            model: snapshot.device.kind.clone(),
            native_name: snapshot.device.name.clone(),
            native_handle: snapshot.device.native_handle,
        },
        current: snapshot.current.clone(),
        operation_modes: snapshot.operation_modes.clone(),
        stop_options: snapshot.stop_options.clone(),
        filters: snapshot.filters.clone(),
        threshold: snapshot.threshold.clone(),
        channel_modes_by_operation_mode: snapshot.channel_modes_by_operation_mode.clone(),
    }
}

pub fn render_device_options_text(response: &DeviceOptionsResponse) -> String {
    let mut lines = vec![
        "device".to_string(),
        format!("  selected_handle: {}", response.selected_handle),
        format!("  stable_id: {}", response.device.stable_id),
        format!("  model: {}", response.device.model),
        format!("  native_name: {}", response.device.native_name),
        format!("  native_handle: {}", response.device.native_handle),
        "operation_modes".to_string(),
        render_current_line(
            response.current.operation_mode_id.as_deref(),
            response.current.operation_mode_code,
        ),
    ];

    append_enum_options(&mut lines, &response.operation_modes);

    lines.push("stop_options".to_string());
    lines.push(render_current_line(
        response.current.stop_option_id.as_deref(),
        response.current.stop_option_code,
    ));
    append_enum_options(&mut lines, &response.stop_options);

    lines.push("filters".to_string());
    lines.push(render_current_line(
        response.current.filter_id.as_deref(),
        response.current.filter_code,
    ));
    append_enum_options(&mut lines, &response.filters);

    lines.push("threshold".to_string());
    lines.push(format!("  id: {}", response.threshold.id));
    lines.push(format!("  kind: {}", response.threshold.kind));
    lines.push(format!(
        "  current_volts: {}",
        format_optional_decimal(response.threshold.current_volts)
    ));
    lines.push(format!(
        "  min_volts: {}",
        format_decimal(response.threshold.min_volts)
    ));
    lines.push(format!(
        "  max_volts: {}",
        format_decimal(response.threshold.max_volts)
    ));
    lines.push(format!(
        "  step_volts: {}",
        format_decimal(response.threshold.step_volts)
    ));

    match &response.threshold.legacy_metadata {
        Some(legacy) => {
            lines.push(format!(
                "  legacy_current_native_code: {}",
                format_optional_code(legacy.current_native_code)
            ));
            for option in &legacy.options {
                lines.push(format!(
                    "  legacy_option: native_code={} | label={}",
                    option.native_code, option.label
                ));
            }
        }
        None => {
            lines.push("  legacy_current_native_code: none".to_string());
        }
    }

    lines.push("channel_modes_by_operation_mode".to_string());
    for group in &response.channel_modes_by_operation_mode {
        lines.push(format!(
            "  group: {} | native_code={} | current_channel_mode={} | current_native_code={}",
            group.operation_mode_id,
            group.operation_mode_code,
            group.current_channel_mode_id.as_deref().unwrap_or("none"),
            format_optional_code(group.current_channel_mode_code)
        ));
        for mode in &group.channel_modes {
            lines.push(format!(
                "  channel_mode: {} | native_code={} | label={} | max_enabled_channels={}",
                mode.id, mode.native_code, mode.label, mode.max_enabled_channels
            ));
        }
    }

    lines.join("\n")
}

fn append_enum_options(lines: &mut Vec<String>, options: &[EnumOptionSnapshot]) {
    for option in options {
        lines.push(format!(
            "  option: {} | native_code={} | label={}",
            option.id, option.native_code, option.label
        ));
    }
}

fn render_current_line(id: Option<&str>, native_code: Option<i16>) -> String {
    format!(
        "  current: {} | native_code={}",
        id.unwrap_or("none"),
        format_optional_code(native_code)
    )
}

fn format_optional_code(value: Option<i16>) -> String {
    value
        .map(|code| code.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_decimal(value: Option<f64>) -> String {
    value
        .map(format_decimal)
        .unwrap_or_else(|| "none".to_string())
}

fn format_decimal(value: f64) -> String {
    let mut formatted = value.to_string();
    if !formatted.contains('.') {
        formatted.push_str(".0");
    }
    formatted
}

#[cfg(test)]
mod tests {
    use super::{build_device_options_response, render_device_options_text};
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

    #[test]
    fn build_device_options_response_surfaces_capture_tokens_and_flags() {
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
                    "native_code": 0,
                    "label": "Buffer Mode"
                },
                {
                    "token": "stream",
                    "stable_id": "operation-mode:1",
                    "native_code": 1,
                    "label": "Stream Mode"
                }
            ],
            "stop_options": [
                {
                    "token": "immediate",
                    "stable_id": "stop-option:0",
                    "native_code": 0,
                    "label": "Immediate"
                },
                {
                    "token": "stop-after-samples",
                    "stable_id": "stop-option:1",
                    "native_code": 1,
                    "label": "Stop after samples"
                }
            ],
            "filters": [
                {
                    "token": "off",
                    "stable_id": "filter:1",
                    "native_code": 1,
                    "label": "Off"
                },
                {
                    "token": "1-sample",
                    "stable_id": "filter:2",
                    "native_code": 2,
                    "label": "1 Sample"
                }
            ],
            "threshold": {
                "threshold_flag": "--threshold-volts",
                "stable_id": "threshold:vth-range",
                "kind": "voltage-range",
                "current_volts": 1.8,
                "min_volts": 0.7,
                "max_volts": 5.0,
                "step_volts": 0.1
            },
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
                            "native_code": 20,
                            "label": "Buffer 100x16",
                            "max_enabled_channels": 16
                        },
                        {
                            "token": "buffer-200x8",
                            "stable_id": "channel-mode:21",
                            "native_code": 21,
                            "label": "Buffer 200x8",
                            "max_enabled_channels": 8
                        }
                    ]
                }
            ],
            "capture_guide": {
                "operation_mode_flag": "--operation-mode",
                "stop_option_flag": "--stop-option",
                "channel_mode_flag": "--channel-mode",
                "threshold_flag": "--threshold-volts",
                "filter_flag": "--filter",
                "channels_flag": "--channels"
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn render_device_options_text_surfaces_copy_paste_capture_flags() {
        let response = build_device_options_response(&sample_snapshot());
        let rendered = render_device_options_text(&response);

        assert!(rendered.contains("--operation-mode buffer"));
        assert!(rendered.contains("--stop-option stop-after-samples"));
        assert!(rendered.contains("--channel-mode buffer-100x16"));
        assert!(rendered.contains("--threshold-volts 1.8"));
        assert!(rendered.contains("--filter off"));
        assert!(rendered.contains("--channels IDX[,IDX...]"));
    }
}
