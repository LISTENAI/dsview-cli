use std::collections::{BTreeMap, BTreeSet};

use dsview_core::{
    ChannelModeOptionSnapshot, DeviceOptionValidationCapabilities, DeviceOptionValidationRequest,
    DeviceOptionsSnapshot, EnumOptionSnapshot,
};
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

pub trait CaptureDeviceOptionInput {
    fn operation_mode(&self) -> Option<&str>;
    fn stop_option(&self) -> Option<&str>;
    fn channel_mode(&self) -> Option<&str>;
    fn threshold_volts(&self) -> Option<f64>;
    fn filter(&self) -> Option<&str>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureDeviceOptionParseError {
    UnsupportedOperationModeToken { token: String },
    UnsupportedStopOptionToken { token: String },
    UnsupportedChannelModeToken { token: String },
    UnsupportedFilterToken { token: String },
    AmbiguousChannelModeToken { token: String },
    MissingCurrentOperationMode,
    MissingCurrentChannelMode,
    ConflictingOperationModeInference { sources: Vec<&'static str> },
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

pub fn resolve_capture_device_option_request<T: CaptureDeviceOptionInput>(
    snapshot: &DeviceOptionsSnapshot,
    capabilities: &DeviceOptionValidationCapabilities,
    args: &T,
    sample_rate_hz: u64,
    sample_limit: u64,
    channels: &[u16],
) -> Result<DeviceOptionValidationRequest, CaptureDeviceOptionParseError> {
    let lookup = token_lookup_maps(snapshot);
    let explicit_operation_mode_id = resolve_known_token(
        args.operation_mode(),
        &lookup.operation_modes_by_token,
        |token| CaptureDeviceOptionParseError::UnsupportedOperationModeToken {
            token: token.to_string(),
        },
    )?;
    let explicit_stop_option_id =
        resolve_known_token(args.stop_option(), &lookup.stop_options_by_token, |token| {
            CaptureDeviceOptionParseError::UnsupportedStopOptionToken {
                token: token.to_string(),
            }
        })?;
    let resolved_channel_mode = resolve_channel_mode(
        snapshot,
        &lookup,
        args.channel_mode(),
        explicit_operation_mode_id.as_deref(),
    )?;
    let filter_id = resolve_known_token(args.filter(), &lookup.filters_by_token, |token| {
        CaptureDeviceOptionParseError::UnsupportedFilterToken {
            token: token.to_string(),
        }
    })?
    .or_else(|| snapshot.current.filter_id.clone());
    let threshold_volts = args.threshold_volts().or(snapshot.threshold.current_volts);
    let operation_mode_id = resolve_operation_mode_id(
        snapshot,
        capabilities,
        explicit_operation_mode_id,
        resolved_channel_mode
            .as_ref()
            .and_then(|mode| mode.inferred_operation_mode_id.clone()),
        args.stop_option()
            .is_some()
            .then_some(explicit_stop_option_id.as_deref())
            .flatten(),
    )?;
    let stop_option_id = explicit_stop_option_id.or_else(|| {
        if snapshot.current.operation_mode_id.as_deref() == Some(operation_mode_id.as_str()) {
            snapshot.current.stop_option_id.clone()
        } else {
            None
        }
    });
    let channel_mode_id = resolved_channel_mode
        .as_ref()
        .map(|mode| mode.stable_id.clone())
        .or_else(|| {
            if snapshot.current.operation_mode_id.as_deref() == Some(operation_mode_id.as_str()) {
                snapshot.current.channel_mode_id.clone()
            } else {
                snapshot
                    .channel_modes_by_operation_mode
                    .iter()
                    .find(|group| group.operation_mode_id == operation_mode_id)
                    .and_then(|group| group.channel_modes.first())
                    .map(|mode| mode.id.clone())
            }
        })
        .ok_or(CaptureDeviceOptionParseError::MissingCurrentChannelMode)?;

    Ok(DeviceOptionValidationRequest {
        operation_mode_id,
        stop_option_id,
        channel_mode_id,
        sample_rate_hz,
        sample_limit,
        enabled_channels: channels.iter().copied().collect::<BTreeSet<_>>(),
        threshold_volts,
        filter_id,
    })
}

fn resolve_known_token<F>(
    token: Option<&str>,
    stable_ids_by_token: &BTreeMap<String, String>,
    error_for_token: F,
) -> Result<Option<String>, CaptureDeviceOptionParseError>
where
    F: FnOnce(&str) -> CaptureDeviceOptionParseError + Copy,
{
    token
        .map(|value| {
            stable_ids_by_token
                .get(value)
                .cloned()
                .ok_or_else(|| error_for_token(value))
        })
        .transpose()
}

fn resolve_operation_mode_id(
    snapshot: &DeviceOptionsSnapshot,
    capabilities: &DeviceOptionValidationCapabilities,
    explicit_operation_mode_id: Option<String>,
    inferred_from_channel_mode: Option<String>,
    stop_option_id: Option<&str>,
) -> Result<String, CaptureDeviceOptionParseError> {
    if let Some(operation_mode_id) = explicit_operation_mode_id {
        return Ok(operation_mode_id);
    }

    let inferred_from_stop_option =
        infer_operation_mode_from_stop_option(capabilities, stop_option_id);
    let mut inferred_operation_modes = BTreeSet::new();
    let mut sources = Vec::new();

    if let Some(operation_mode_id) = inferred_from_channel_mode {
        inferred_operation_modes.insert(operation_mode_id);
        sources.push("channel_mode");
    }
    if let Some(operation_mode_id) = inferred_from_stop_option {
        inferred_operation_modes.insert(operation_mode_id);
        sources.push("stop_option");
    }

    match inferred_operation_modes.len() {
        0 => snapshot
            .current
            .operation_mode_id
            .clone()
            .ok_or(CaptureDeviceOptionParseError::MissingCurrentOperationMode),
        1 => Ok(inferred_operation_modes.into_iter().next().unwrap()),
        _ => Err(CaptureDeviceOptionParseError::ConflictingOperationModeInference { sources }),
    }
}

fn infer_operation_mode_from_stop_option(
    capabilities: &DeviceOptionValidationCapabilities,
    stop_option_id: Option<&str>,
) -> Option<String> {
    let stop_option_id = stop_option_id?;
    let mut matches = capabilities
        .operation_modes
        .iter()
        .filter(|operation_mode| {
            operation_mode
                .stop_option_ids
                .iter()
                .any(|supported_stop_option_id| supported_stop_option_id == stop_option_id)
        })
        .map(|operation_mode| operation_mode.id.clone());

    let first = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(first)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedChannelMode {
    stable_id: String,
    inferred_operation_mode_id: Option<String>,
}

fn resolve_channel_mode(
    snapshot: &DeviceOptionsSnapshot,
    lookup: &CaptureTokenLookupMaps,
    channel_mode_token: Option<&str>,
    explicit_operation_mode_id: Option<&str>,
) -> Result<Option<ResolvedChannelMode>, CaptureDeviceOptionParseError> {
    let Some(channel_mode_token) = channel_mode_token else {
        return Ok(None);
    };

    let matches = snapshot
        .channel_modes_by_operation_mode
        .iter()
        .flat_map(|group| {
            group.channel_modes.iter().filter_map(|channel_mode| {
                let token = lookup
                    .channel_mode_tokens_by_stable_id
                    .get(&channel_mode.id)?;
                (token == channel_mode_token)
                    .then(|| (group.operation_mode_id.clone(), channel_mode.id.clone()))
            })
        })
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(CaptureDeviceOptionParseError::UnsupportedChannelModeToken {
            token: channel_mode_token.to_string(),
        });
    }

    let mut stable_ids = matches
        .iter()
        .map(|(_, stable_id)| stable_id.clone())
        .collect::<BTreeSet<_>>();
    if stable_ids.len() == 1 {
        return Ok(Some(ResolvedChannelMode {
            stable_id: stable_ids.pop_first().unwrap(),
            inferred_operation_mode_id: infer_unique_operation_mode(&matches),
        }));
    }

    if let Some(explicit_operation_mode_id) = explicit_operation_mode_id {
        let explicit_matches = matches
            .iter()
            .filter(|(operation_mode_id, _)| operation_mode_id == explicit_operation_mode_id)
            .map(|(_, stable_id)| stable_id.clone())
            .collect::<BTreeSet<_>>();
        if explicit_matches.len() == 1 {
            return Ok(Some(ResolvedChannelMode {
                stable_id: explicit_matches.into_iter().next().unwrap(),
                inferred_operation_mode_id: None,
            }));
        }
    }

    Err(CaptureDeviceOptionParseError::AmbiguousChannelModeToken {
        token: channel_mode_token.to_string(),
    })
}

fn infer_unique_operation_mode(matches: &[(String, String)]) -> Option<String> {
    let mut operation_mode_ids = matches
        .iter()
        .map(|(operation_mode_id, _)| operation_mode_id.clone())
        .collect::<BTreeSet<_>>();
    if operation_mode_ids.len() == 1 {
        operation_mode_ids.pop_first()
    } else {
        None
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
