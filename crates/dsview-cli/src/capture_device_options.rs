use std::collections::BTreeMap;

use dsview_core::{ChannelModeOptionSnapshot, DeviceOptionsSnapshot, EnumOptionSnapshot};
use serde::Serialize;

const BUFFER_TOKEN: &str = "buffer";
const STREAM_TOKEN: &str = "stream";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CliTokenOption {
    pub token: String,
    pub stable_id: String,
    pub native_code: i16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CliChannelModeOption {
    pub token: String,
    pub stable_id: String,
    pub native_code: i16,
    pub label: String,
    pub max_enabled_channels: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureTokenGuide {
    pub operation_mode_flag: String,
    pub stop_option_flag: String,
    pub channel_mode_flag: String,
    pub threshold_flag: String,
    pub filter_flag: String,
    pub channels_flag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTokenLookupMaps {
    pub operation_modes_by_token: BTreeMap<String, String>,
    pub operation_mode_tokens_by_stable_id: BTreeMap<String, String>,
    pub stop_options_by_token: BTreeMap<String, String>,
    pub stop_option_tokens_by_stable_id: BTreeMap<String, String>,
    pub filters_by_token: BTreeMap<String, String>,
    pub filter_tokens_by_stable_id: BTreeMap<String, String>,
    pub channel_modes_by_token: BTreeMap<String, String>,
    pub channel_mode_tokens_by_stable_id: BTreeMap<String, String>,
    pub channel_mode_parent_operation_modes: BTreeMap<String, String>,
}

pub fn slug_token(label: &str) -> String {
    let mut token = String::new();
    let mut previous_was_dash = false;

    for character in label.chars() {
        let normalized = character.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            token.push(normalized);
            previous_was_dash = false;
        } else if !previous_was_dash && !token.is_empty() {
            token.push('-');
            previous_was_dash = true;
        }
    }

    while token.ends_with('-') {
        token.pop();
    }

    token
}

pub fn build_capture_token_guide() -> CaptureTokenGuide {
    CaptureTokenGuide {
        operation_mode_flag: "--operation-mode".to_string(),
        stop_option_flag: "--stop-option".to_string(),
        channel_mode_flag: "--channel-mode".to_string(),
        threshold_flag: "--threshold-volts".to_string(),
        filter_flag: "--filter".to_string(),
        channels_flag: "--channels".to_string(),
    }
}

pub fn token_lookup_maps(snapshot: &DeviceOptionsSnapshot) -> CaptureTokenLookupMaps {
    let mut lookup = CaptureTokenLookupMaps {
        operation_modes_by_token: BTreeMap::new(),
        operation_mode_tokens_by_stable_id: BTreeMap::new(),
        stop_options_by_token: BTreeMap::new(),
        stop_option_tokens_by_stable_id: BTreeMap::new(),
        filters_by_token: BTreeMap::new(),
        filter_tokens_by_stable_id: BTreeMap::new(),
        channel_modes_by_token: BTreeMap::new(),
        channel_mode_tokens_by_stable_id: BTreeMap::new(),
        channel_mode_parent_operation_modes: BTreeMap::new(),
    };

    for option in &snapshot.operation_modes {
        let option = build_operation_mode_option(option);
        lookup
            .operation_modes_by_token
            .insert(option.token.clone(), option.stable_id.clone());
        lookup
            .operation_mode_tokens_by_stable_id
            .insert(option.stable_id, option.token);
    }

    for option in &snapshot.stop_options {
        let option = build_cli_token_option(option);
        lookup
            .stop_options_by_token
            .insert(option.token.clone(), option.stable_id.clone());
        lookup
            .stop_option_tokens_by_stable_id
            .insert(option.stable_id, option.token);
    }

    for option in &snapshot.filters {
        let option = build_cli_token_option(option);
        lookup
            .filters_by_token
            .insert(option.token.clone(), option.stable_id.clone());
        lookup
            .filter_tokens_by_stable_id
            .insert(option.stable_id, option.token);
    }

    for group in &snapshot.channel_modes_by_operation_mode {
        for option in &group.channel_modes {
            let option = build_cli_channel_mode_option(option);
            lookup
                .channel_mode_parent_operation_modes
                .insert(option.token.clone(), group.operation_mode_id.clone());
            lookup
                .channel_modes_by_token
                .insert(option.token.clone(), option.stable_id.clone());
            lookup
                .channel_mode_tokens_by_stable_id
                .insert(option.stable_id, option.token);
        }
    }

    lookup
}

pub(crate) fn build_operation_mode_option(option: &EnumOptionSnapshot) -> CliTokenOption {
    CliTokenOption {
        token: operation_mode_token(&option.label),
        stable_id: option.id.clone(),
        native_code: option.native_code,
        label: option.label.clone(),
    }
}

pub(crate) fn build_cli_token_option(option: &EnumOptionSnapshot) -> CliTokenOption {
    CliTokenOption {
        token: slug_token(&option.label),
        stable_id: option.id.clone(),
        native_code: option.native_code,
        label: option.label.clone(),
    }
}

pub(crate) fn build_cli_channel_mode_option(
    option: &ChannelModeOptionSnapshot,
) -> CliChannelModeOption {
    CliChannelModeOption {
        token: slug_token(&option.label),
        stable_id: option.id.clone(),
        native_code: option.native_code,
        label: option.label.clone(),
        max_enabled_channels: option.max_enabled_channels,
    }
}

fn operation_mode_token(label: &str) -> String {
    match slug_token(label).as_str() {
        BUFFER_TOKEN | "buffer-mode" => BUFFER_TOKEN.to_string(),
        STREAM_TOKEN | "stream-mode" => STREAM_TOKEN.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_capture_token_guide, build_cli_channel_mode_option, build_cli_token_option,
        build_operation_mode_option, slug_token,
    };
    use dsview_core::{ChannelModeOptionSnapshot, EnumOptionSnapshot};

    #[test]
    fn slug_token_normalizes_display_labels_to_kebab_case() {
        assert_eq!(slug_token("Stop after samples"), "stop-after-samples");
        assert_eq!(slug_token("Buffer 100x16"), "buffer-100x16");
        assert_eq!(slug_token("1 Sample"), "1-sample");
    }

    #[test]
    fn operation_mode_tokens_use_short_capture_aliases() {
        let buffer = build_operation_mode_option(&EnumOptionSnapshot {
            id: "operation-mode:0".to_string(),
            native_code: 0,
            label: "Buffer Mode".to_string(),
        });
        let stream = build_operation_mode_option(&EnumOptionSnapshot {
            id: "operation-mode:1".to_string(),
            native_code: 1,
            label: "Stream Mode".to_string(),
        });

        assert_eq!(buffer.token, "buffer");
        assert_eq!(stream.token, "stream");
    }

    #[test]
    fn channel_mode_tokens_keep_channel_limits() {
        let option = build_cli_channel_mode_option(&ChannelModeOptionSnapshot {
            id: "channel-mode:20".to_string(),
            native_code: 20,
            label: "Buffer 100x16".to_string(),
            max_enabled_channels: 16,
        });

        assert_eq!(option.token, "buffer-100x16");
        assert_eq!(option.max_enabled_channels, 16);
    }

    #[test]
    fn capture_token_guide_uses_capture_flag_names() {
        let guide = build_capture_token_guide();

        assert_eq!(guide.operation_mode_flag, "--operation-mode");
        assert_eq!(guide.stop_option_flag, "--stop-option");
        assert_eq!(guide.channel_mode_flag, "--channel-mode");
        assert_eq!(guide.threshold_flag, "--threshold-volts");
        assert_eq!(guide.filter_flag, "--filter");
        assert_eq!(guide.channels_flag, "--channels");
    }

    #[test]
    fn enum_tokens_reuse_kebab_case_normalization() {
        let option = build_cli_token_option(&EnumOptionSnapshot {
            id: "stop-option:1".to_string(),
            native_code: 1,
            label: "Stop after samples".to_string(),
        });

        assert_eq!(option.token, "stop-after-samples");
    }
}
