use dsview_sys::{
    DeviceOptionChannelMode, DeviceOptionChannelModeGroup, DeviceOptionValue,
    DeviceOptionsSnapshot as NativeDeviceOptionsSnapshot,
    LegacyThresholdMetadata as NativeLegacyThresholdMetadata,
};
use serde::Serialize;

use crate::SupportedDevice;

const OPERATION_MODE_PREFIX: &str = "operation-mode";
const STOP_OPTION_PREFIX: &str = "stop-option";
const FILTER_PREFIX: &str = "filter";
const CHANNEL_MODE_PREFIX: &str = "channel-mode";
const THRESHOLD_ID: &str = "threshold:vth-range";
const THRESHOLD_KIND: &str = "voltage-range";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceIdentitySnapshot {
    pub selection_handle: u64,
    pub native_handle: u64,
    pub stable_id: String,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnumOptionSnapshot {
    pub id: String,
    pub native_code: i16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelModeOptionSnapshot {
    pub id: String,
    pub native_code: i16,
    pub label: String,
    pub max_enabled_channels: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurrentDeviceOptionValues {
    pub operation_mode_id: Option<String>,
    pub operation_mode_code: Option<i16>,
    pub stop_option_id: Option<String>,
    pub stop_option_code: Option<i16>,
    pub filter_id: Option<String>,
    pub filter_code: Option<i16>,
    pub channel_mode_id: Option<String>,
    pub channel_mode_code: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelModeGroupSnapshot {
    pub operation_mode_id: String,
    pub operation_mode_code: i16,
    pub current_channel_mode_id: Option<String>,
    pub current_channel_mode_code: Option<i16>,
    pub channel_modes: Vec<ChannelModeOptionSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RawOptionMetadataSnapshot {
    pub native_code: i16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LegacyThresholdMetadataSnapshot {
    pub current_native_code: Option<i16>,
    pub options: Vec<RawOptionMetadataSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThresholdCapabilitySnapshot {
    pub id: String,
    pub kind: String,
    pub current_volts: Option<f64>,
    pub min_volts: f64,
    pub max_volts: f64,
    pub step_volts: f64,
    pub legacy_metadata: Option<LegacyThresholdMetadataSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceOptionsSnapshot {
    pub device: DeviceIdentitySnapshot,
    pub current: CurrentDeviceOptionValues,
    pub operation_modes: Vec<EnumOptionSnapshot>,
    pub stop_options: Vec<EnumOptionSnapshot>,
    pub filters: Vec<EnumOptionSnapshot>,
    pub channel_modes_by_operation_mode: Vec<ChannelModeGroupSnapshot>,
    pub threshold: ThresholdCapabilitySnapshot,
}

pub fn normalize_device_options_snapshot(
    device: &SupportedDevice,
    native: NativeDeviceOptionsSnapshot,
) -> DeviceOptionsSnapshot {
    let NativeDeviceOptionsSnapshot {
        current_operation_mode_code,
        operation_modes,
        current_stop_option_code,
        stop_options,
        current_filter_code,
        filters,
        current_channel_mode_code,
        channel_mode_groups,
        threshold,
    } = native;

    DeviceOptionsSnapshot {
        device: DeviceIdentitySnapshot {
            selection_handle: device.selection_handle.raw(),
            native_handle: device.native_handle.raw(),
            stable_id: device.stable_id.to_string(),
            kind: device.kind.display_name().to_string(),
            name: device.name.clone(),
        },
        current: CurrentDeviceOptionValues {
            operation_mode_id: current_operation_mode_code.map(operation_mode_id),
            operation_mode_code: current_operation_mode_code,
            stop_option_id: current_stop_option_code.map(stop_option_id),
            stop_option_code: current_stop_option_code,
            filter_id: current_filter_code.map(filter_id),
            filter_code: current_filter_code,
            channel_mode_id: current_channel_mode_code.map(channel_mode_id),
            channel_mode_code: current_channel_mode_code,
        },
        operation_modes: sort_enum_options(operation_modes, operation_mode_id),
        stop_options: sort_enum_options(stop_options, stop_option_id),
        filters: sort_enum_options(filters, filter_id),
        channel_modes_by_operation_mode: sort_channel_mode_groups(
            channel_mode_groups,
            current_operation_mode_code,
            current_channel_mode_code,
        ),
        threshold: ThresholdCapabilitySnapshot {
            id: THRESHOLD_ID.to_string(),
            kind: THRESHOLD_KIND.to_string(),
            current_volts: threshold.current_volts,
            min_volts: threshold.min_volts,
            max_volts: threshold.max_volts,
            step_volts: threshold.step_volts,
            legacy_metadata: threshold.legacy.map(normalize_legacy_threshold_metadata),
        },
    }
}

fn sort_enum_options(
    mut options: Vec<DeviceOptionValue>,
    id_for_code: fn(i16) -> String,
) -> Vec<EnumOptionSnapshot> {
    options.sort_by_key(|option| option.code);
    options
        .into_iter()
        .map(|option| EnumOptionSnapshot {
            id: id_for_code(option.code),
            native_code: option.code,
            label: option.label,
        })
        .collect()
}

fn sort_channel_mode_groups(
    mut groups: Vec<DeviceOptionChannelModeGroup>,
    current_operation_mode_code: Option<i16>,
    current_channel_mode_code: Option<i16>,
) -> Vec<ChannelModeGroupSnapshot> {
    groups.sort_by_key(|group| group.operation_mode_code);
    groups
        .into_iter()
        .map(|group| normalize_channel_mode_group(group, current_operation_mode_code, current_channel_mode_code))
        .collect()
}

fn normalize_channel_mode_group(
    mut group: DeviceOptionChannelModeGroup,
    current_operation_mode_code: Option<i16>,
    current_channel_mode_code: Option<i16>,
) -> ChannelModeGroupSnapshot {
    group.channel_modes.sort_by_key(|mode| mode.code);
    let group_current_channel_mode_code = if current_operation_mode_code == Some(group.operation_mode_code) {
        current_channel_mode_code
    } else {
        None
    };

    ChannelModeGroupSnapshot {
        operation_mode_id: operation_mode_id(group.operation_mode_code),
        operation_mode_code: group.operation_mode_code,
        current_channel_mode_id: group_current_channel_mode_code.map(channel_mode_id),
        current_channel_mode_code: group_current_channel_mode_code,
        channel_modes: group
            .channel_modes
            .into_iter()
            .map(normalize_channel_mode)
            .collect(),
    }
}

fn normalize_channel_mode(mode: DeviceOptionChannelMode) -> ChannelModeOptionSnapshot {
    ChannelModeOptionSnapshot {
        id: channel_mode_id(mode.code),
        native_code: mode.code,
        label: mode.label,
        max_enabled_channels: mode.max_enabled_channels,
    }
}

fn normalize_legacy_threshold_metadata(
    mut legacy: NativeLegacyThresholdMetadata,
) -> LegacyThresholdMetadataSnapshot {
    legacy.options.sort_by_key(|option| option.code);
    LegacyThresholdMetadataSnapshot {
        current_native_code: legacy.current_code,
        options: legacy
            .options
            .into_iter()
            .map(|option| RawOptionMetadataSnapshot {
                native_code: option.code,
                label: option.label,
            })
            .collect(),
    }
}

fn operation_mode_id(code: i16) -> String {
    format!("{OPERATION_MODE_PREFIX}:{code}")
}

fn stop_option_id(code: i16) -> String {
    format!("{STOP_OPTION_PREFIX}:{code}")
}

fn filter_id(code: i16) -> String {
    format!("{FILTER_PREFIX}:{code}")
}

fn channel_mode_id(code: i16) -> String {
    format!("{CHANNEL_MODE_PREFIX}:{code}")
}
