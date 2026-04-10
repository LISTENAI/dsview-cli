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
