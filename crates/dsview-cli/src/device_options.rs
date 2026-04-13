use std::collections::BTreeMap;

use crate::capture_device_options::{
    CaptureTokenGuide, CliChannelModeOption, CliTokenOption, build_capture_token_guide,
    build_cli_channel_mode_option, build_cli_token_option, build_operation_mode_option,
    token_lookup_maps,
};
use dsview_core::{
    CurrentDeviceOptionValues, DeviceOptionsSnapshot, LegacyThresholdMetadataSnapshot,
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
pub struct CurrentCaptureOptionValues {
    pub operation_mode_token: Option<String>,
    pub operation_mode_stable_id: Option<String>,
    pub operation_mode_code: Option<i16>,
    pub stop_option_token: Option<String>,
    pub stop_option_stable_id: Option<String>,
    pub stop_option_code: Option<i16>,
    pub filter_token: Option<String>,
    pub filter_stable_id: Option<String>,
    pub filter_code: Option<i16>,
    pub channel_mode_token: Option<String>,
    pub channel_mode_stable_id: Option<String>,
    pub channel_mode_code: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelModeGroupResponse {
    pub operation_mode_token: String,
    pub operation_mode_stable_id: String,
    pub operation_mode_code: i16,
    pub current_channel_mode_token: Option<String>,
    pub current_channel_mode_stable_id: Option<String>,
    pub current_channel_mode_code: Option<i16>,
    pub channel_modes: Vec<CliChannelModeOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThresholdGuideResponse {
    pub threshold_flag: String,
    pub stable_id: String,
    pub kind: String,
    pub current_volts: Option<f64>,
    pub min_volts: f64,
    pub max_volts: f64,
    pub step_volts: f64,
    pub legacy_metadata: Option<LegacyThresholdMetadataSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceOptionsResponse {
    pub selected_handle: u64,
    pub device: DeviceIdentityResponse,
    pub current: CurrentCaptureOptionValues,
    pub operation_modes: Vec<CliTokenOption>,
    pub stop_options: Vec<CliTokenOption>,
    pub filters: Vec<CliTokenOption>,
    pub threshold: ThresholdGuideResponse,
    pub channel_modes_by_operation_mode: Vec<ChannelModeGroupResponse>,
    pub capture_guide: CaptureTokenGuide,
}

pub fn build_device_options_response(snapshot: &DeviceOptionsSnapshot) -> DeviceOptionsResponse {
    let capture_guide = build_capture_token_guide();
    let lookup_maps = token_lookup_maps(snapshot);

    DeviceOptionsResponse {
        selected_handle: snapshot.device.selection_handle,
        device: DeviceIdentityResponse {
            stable_id: snapshot.device.stable_id.clone(),
            model: snapshot.device.kind.clone(),
            native_name: snapshot.device.name.clone(),
            native_handle: snapshot.device.native_handle,
        },
        current: build_current_values(&snapshot.current, &lookup_maps),
        operation_modes: snapshot
            .operation_modes
            .iter()
            .map(build_operation_mode_option)
            .collect(),
        stop_options: snapshot
            .stop_options
            .iter()
            .map(build_cli_token_option)
            .collect(),
        filters: snapshot
            .filters
            .iter()
            .map(build_cli_token_option)
            .collect(),
        threshold: ThresholdGuideResponse {
            threshold_flag: capture_guide.threshold_flag.clone(),
            stable_id: snapshot.threshold.id.clone(),
            kind: snapshot.threshold.kind.clone(),
            current_volts: snapshot.threshold.current_volts,
            min_volts: snapshot.threshold.min_volts,
            max_volts: snapshot.threshold.max_volts,
            step_volts: snapshot.threshold.step_volts,
            legacy_metadata: snapshot.threshold.legacy_metadata.clone(),
        },
        channel_modes_by_operation_mode: snapshot
            .channel_modes_by_operation_mode
            .iter()
            .map(|group| ChannelModeGroupResponse {
                operation_mode_token: lookup_maps
                    .operation_mode_tokens_by_stable_id
                    .get(&group.operation_mode_id)
                    .cloned()
                    .unwrap_or_else(|| group.operation_mode_id.clone()),
                operation_mode_stable_id: group.operation_mode_id.clone(),
                operation_mode_code: group.operation_mode_code,
                current_channel_mode_token: lookup_current_token(
                    group.current_channel_mode_id.as_deref(),
                    &lookup_maps.channel_mode_tokens_by_stable_id,
                ),
                current_channel_mode_stable_id: group.current_channel_mode_id.clone(),
                current_channel_mode_code: group.current_channel_mode_code,
                channel_modes: group
                    .channel_modes
                    .iter()
                    .map(build_cli_channel_mode_option)
                    .collect(),
            })
            .collect(),
        capture_guide,
    }
}

pub fn render_device_options_text(response: &DeviceOptionsResponse) -> String {
    let current_threshold = format_optional_decimal(response.threshold.current_volts);
    let mut lines = vec![
        "device".to_string(),
        format!("  selected_handle: {}", response.selected_handle),
        format!("  stable_id: {}", response.device.stable_id),
        format!("  model: {}", response.device.model),
        format!("  native_name: {}", response.device.native_name),
        format!("  native_handle: {}", response.device.native_handle),
        "capture_flags".to_string(),
        render_capture_flag_line(
            &response.capture_guide.operation_mode_flag,
            response.current.operation_mode_token.as_deref(),
        ),
        render_capture_flag_line(
            &response.capture_guide.stop_option_flag,
            response.current.stop_option_token.as_deref(),
        ),
        render_capture_flag_line(
            &response.capture_guide.channel_mode_flag,
            response.current.channel_mode_token.as_deref(),
        ),
        render_capture_flag_value_line(&response.capture_guide.threshold_flag, &current_threshold),
        render_capture_flag_line(
            &response.capture_guide.filter_flag,
            response.current.filter_token.as_deref(),
        ),
        render_channels_flag_line(response),
        "operation_modes".to_string(),
        render_current_token_line(
            response.current.operation_mode_token.as_deref(),
            response.current.operation_mode_stable_id.as_deref(),
            response.current.operation_mode_code,
        ),
    ];

    append_token_options(&mut lines, &response.operation_modes);

    lines.push("stop_options".to_string());
    lines.push(render_current_token_line(
        response.current.stop_option_token.as_deref(),
        response.current.stop_option_stable_id.as_deref(),
        response.current.stop_option_code,
    ));
    append_token_options(&mut lines, &response.stop_options);

    lines.push("channel_modes_by_operation_mode".to_string());
    for group in &response.channel_modes_by_operation_mode {
        lines.push(format!(
            "  group: {} | stable_id={} | native_code={} | current_channel_mode={} | current_stable_id={} | current_native_code={}",
            group.operation_mode_token,
            group.operation_mode_stable_id,
            group.operation_mode_code,
            group.current_channel_mode_token.as_deref().unwrap_or("none"),
            group.current_channel_mode_stable_id.as_deref().unwrap_or("none"),
            format_optional_code(group.current_channel_mode_code),
        ));
        for mode in &group.channel_modes {
            lines.push(format!(
                "  channel_mode: {} | stable_id={} | native_code={} | label={} | max_enabled_channels={}",
                mode.token, mode.stable_id, mode.native_code, mode.label, mode.max_enabled_channels
            ));
        }
    }

    lines.push("threshold".to_string());
    lines.push(render_capture_flag_value_line(
        &response.threshold.threshold_flag,
        &current_threshold,
    ));
    lines.push(format!("  stable_id: {}", response.threshold.stable_id));
    lines.push(format!("  kind: {}", response.threshold.kind));
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
        None => lines.push("  legacy_current_native_code: none".to_string()),
    }

    lines.push("filters".to_string());
    lines.push(render_current_token_line(
        response.current.filter_token.as_deref(),
        response.current.filter_stable_id.as_deref(),
        response.current.filter_code,
    ));
    append_token_options(&mut lines, &response.filters);

    lines.join("\n")
}

fn build_current_values(
    current: &CurrentDeviceOptionValues,
    lookup_maps: &crate::capture_device_options::CaptureTokenLookupMaps,
) -> CurrentCaptureOptionValues {
    CurrentCaptureOptionValues {
        operation_mode_token: lookup_current_token(
            current.operation_mode_id.as_deref(),
            &lookup_maps.operation_mode_tokens_by_stable_id,
        ),
        operation_mode_stable_id: current.operation_mode_id.clone(),
        operation_mode_code: current.operation_mode_code,
        stop_option_token: lookup_current_token(
            current.stop_option_id.as_deref(),
            &lookup_maps.stop_option_tokens_by_stable_id,
        ),
        stop_option_stable_id: current.stop_option_id.clone(),
        stop_option_code: current.stop_option_code,
        filter_token: lookup_current_token(
            current.filter_id.as_deref(),
            &lookup_maps.filter_tokens_by_stable_id,
        ),
        filter_stable_id: current.filter_id.clone(),
        filter_code: current.filter_code,
        channel_mode_token: lookup_current_token(
            current.channel_mode_id.as_deref(),
            &lookup_maps.channel_mode_tokens_by_stable_id,
        ),
        channel_mode_stable_id: current.channel_mode_id.clone(),
        channel_mode_code: current.channel_mode_code,
    }
}

fn lookup_current_token(
    value: Option<&str>,
    tokens_by_stable_id: &BTreeMap<String, String>,
) -> Option<String> {
    value.and_then(|stable_id| tokens_by_stable_id.get(stable_id).cloned())
}

fn append_token_options(lines: &mut Vec<String>, options: &[CliTokenOption]) {
    for option in options {
        lines.push(format!(
            "  option: token={} | stable_id={} | native_code={} | label={}",
            option.token, option.stable_id, option.native_code, option.label
        ));
    }
}

fn render_current_token_line(
    token: Option<&str>,
    stable_id: Option<&str>,
    native_code: Option<i16>,
) -> String {
    format!(
        "  current: {} | stable_id={} | native_code={}",
        token.unwrap_or("none"),
        stable_id.unwrap_or("none"),
        format_optional_code(native_code)
    )
}

fn render_capture_flag_line(flag: &str, value: Option<&str>) -> String {
    format!("  {} {}", flag, value.unwrap_or("none"))
}

fn render_capture_flag_value_line(flag: &str, value: &str) -> String {
    format!("  {} {}", flag, value)
}

fn render_channels_flag_line(response: &DeviceOptionsResponse) -> String {
    let line = format!("  {} IDX[,IDX...]", response.capture_guide.channels_flag);

    match current_channel_mode_limit(response) {
        Some((token, max_enabled_channels)) => format!(
            "{}  ({} allows up to {} enabled channels)",
            line, token, max_enabled_channels
        ),
        None => line,
    }
}

fn current_channel_mode_limit(response: &DeviceOptionsResponse) -> Option<(&str, u16)> {
    let current_stable_id = response.current.channel_mode_stable_id.as_deref()?;

    response
        .channel_modes_by_operation_mode
        .iter()
        .flat_map(|group| group.channel_modes.iter())
        .find(|mode| mode.stable_id == current_stable_id)
        .map(|mode| (mode.token.as_str(), mode.max_enabled_channels))
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

        assert_eq!(actual["current"]["operation_mode_token"], "buffer");
        assert_eq!(actual["current"]["stop_option_token"], "stop-after-samples");
        assert_eq!(actual["current"]["channel_mode_token"], "buffer-100x16");
        assert_eq!(actual["current"]["filter_token"], "off");
        assert_eq!(actual["operation_modes"][0]["token"], "buffer");
        assert_eq!(actual["operation_modes"][1]["token"], "stream");
        assert_eq!(actual["stop_options"][1]["stable_id"], "stop-option:1");
        assert_eq!(actual["filters"][1]["token"], "1-sample");
        assert_eq!(actual["threshold"]["threshold_flag"], "--threshold-volts");
        assert_eq!(actual["threshold"]["min_volts"], 0.7);
        assert_eq!(actual["threshold"]["step_volts"], 0.1);
        assert_eq!(
            actual["threshold"]["legacy_metadata"]["current_native_code"],
            3
        );
        assert_eq!(
            actual["channel_modes_by_operation_mode"][0]["operation_mode_token"],
            "buffer"
        );
        assert_eq!(
            actual["channel_modes_by_operation_mode"][1]["operation_mode_token"],
            "stream"
        );
        assert_eq!(
            actual["channel_modes_by_operation_mode"][0]["channel_modes"][0]["token"],
            "buffer-100x16"
        );
        assert_eq!(
            actual["channel_modes_by_operation_mode"][0]["channel_modes"][0]["max_enabled_channels"],
            16
        );
        assert_eq!(
            actual["capture_guide"]["operation_mode_flag"],
            "--operation-mode"
        );
        assert_eq!(actual["capture_guide"]["channels_flag"], "--channels");
    }

    #[test]
    fn render_device_options_text_surfaces_copy_paste_capture_flags() {
        let response = build_device_options_response(&sample_snapshot());
        let rendered = render_device_options_text(&response);

        assert!(rendered.contains("device\n  selected_handle: 7"));
        assert!(rendered.contains("capture_flags\n  --operation-mode buffer"));
        assert!(rendered.contains("  --stop-option stop-after-samples"));
        assert!(rendered.contains("  --channel-mode buffer-100x16"));
        assert!(rendered.contains("  --threshold-volts 1.8"));
        assert!(rendered.contains("  --filter off"));
        assert!(rendered.contains("  --channels IDX[,IDX...]"));
        assert!(rendered.contains("buffer-100x16 allows up to 16 enabled channels"));
    }
}
