use std::collections::BTreeSet;

use dsview_core::{
    ChannelModeValidationCapabilities, CurrentDeviceOptionValues, DeviceIdentitySnapshot,
    DeviceOptionValidationCapabilities, DeviceOptionValidationError, DeviceOptionValidationRequest,
    EnumOptionSnapshot, OperationModeValidationCapabilities, ThresholdCapabilitySnapshot,
    ValidatedDeviceOptionRequest,
};

fn validation_request() -> DeviceOptionValidationRequest {
    DeviceOptionValidationRequest {
        operation_mode_id: "operation-mode:101".to_string(),
        stop_option_id: Some("stop-option:1".to_string()),
        channel_mode_id: "channel-mode:11".to_string(),
        sample_rate_hz: 100_000_000,
        sample_limit: 4096,
        enabled_channels: [0_u16, 1, 2, 3].into_iter().collect::<BTreeSet<_>>(),
        threshold_volts: Some(1.8),
        filter_id: Some("filter:1".to_string()),
    }
}

fn validation_capabilities() -> DeviceOptionValidationCapabilities {
    DeviceOptionValidationCapabilities {
        device: DeviceIdentitySnapshot {
            selection_handle: 1,
            native_handle: 41,
            stable_id: "dslogic-plus".to_string(),
            kind: "DSLogic Plus".to_string(),
            name: "DSLogic PLus".to_string(),
        },
        current: CurrentDeviceOptionValues {
            operation_mode_id: Some("operation-mode:101".to_string()),
            operation_mode_code: Some(101),
            stop_option_id: Some("stop-option:1".to_string()),
            stop_option_code: Some(1),
            filter_id: Some("filter:1".to_string()),
            filter_code: Some(1),
            channel_mode_id: Some("channel-mode:11".to_string()),
            channel_mode_code: Some(11),
        },
        total_channel_count: 16,
        hardware_sample_capacity: 268_435_456,
        sample_limit_alignment: 1024,
        operation_modes: vec![OperationModeValidationCapabilities {
            id: "operation-mode:101".to_string(),
            native_code: 101,
            label: "Buffer Mode".to_string(),
            stop_option_ids: vec!["stop-option:1".to_string(), "stop-option:2".to_string()],
            channel_modes: vec![ChannelModeValidationCapabilities {
                id: "channel-mode:11".to_string(),
                native_code: 11,
                label: "Buffer wide lanes".to_string(),
                max_enabled_channels: 16,
                supported_sample_rates: vec![50_000_000, 100_000_000],
            }],
        }],
        filters: vec![
            EnumOptionSnapshot {
                id: "filter:0".to_string(),
                native_code: 0,
                label: "No filtering".to_string(),
            },
            EnumOptionSnapshot {
                id: "filter:1".to_string(),
                native_code: 1,
                label: "Single-clock filter".to_string(),
            },
        ],
        threshold: ThresholdCapabilitySnapshot {
            id: "threshold:vth-range".to_string(),
            kind: "voltage-range".to_string(),
            current_volts: Some(1.8),
            min_volts: 0.0,
            max_volts: 5.0,
            step_volts: 0.1,
            legacy_metadata: None,
        },
    }
}

#[test]
fn validation_request_contract_exposes_phase_11_fields() {
    let request = validation_request();

    assert_eq!(request.operation_mode_id, "operation-mode:101");
    assert_eq!(request.stop_option_id.as_deref(), Some("stop-option:1"));
    assert_eq!(request.channel_mode_id, "channel-mode:11");
    assert_eq!(request.sample_rate_hz, 100_000_000);
    assert_eq!(request.sample_limit, 4096);
    assert_eq!(
        request.enabled_channels.into_iter().collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(request.threshold_volts, Some(1.8));
    assert_eq!(request.filter_id.as_deref(), Some("filter:1"));
}

#[test]
fn validation_capability_contract_carries_mode_scoped_facts() {
    let capabilities = validation_capabilities();

    assert_eq!(capabilities.total_channel_count, 16);
    assert_eq!(capabilities.hardware_sample_capacity, 268_435_456);
    assert_eq!(capabilities.sample_limit_alignment, 1024);
    assert_eq!(capabilities.operation_modes[0].stop_option_ids.len(), 2);
    assert_eq!(
        capabilities.operation_modes[0].channel_modes[0].supported_sample_rates,
        vec![50_000_000, 100_000_000]
    );
}

#[test]
fn validated_request_preserves_stable_ids_and_native_codes() {
    let validated = ValidatedDeviceOptionRequest {
        operation_mode_id: "operation-mode:101".to_string(),
        operation_mode_code: 101,
        stop_option_id: Some("stop-option:1".to_string()),
        stop_option_code: Some(1),
        channel_mode_id: "channel-mode:11".to_string(),
        channel_mode_code: 11,
        sample_rate_hz: 100_000_000,
        requested_sample_limit: 4096,
        effective_sample_limit: 4096,
        enabled_channels: vec![0, 1, 2, 3],
        threshold_volts: Some(1.8),
        filter_id: Some("filter:1".to_string()),
        filter_code: Some(1),
    };

    assert_eq!(validated.operation_mode_code, 101);
    assert_eq!(validated.channel_mode_code, 11);
    assert_eq!(validated.stop_option_code, Some(1));
    assert_eq!(validated.filter_code, Some(1));
    assert_eq!(validated.enabled_channels, vec![0, 1, 2, 3]);
}

#[test]
fn validation_error_codes_are_stable() {
    let cases = [
        (
            DeviceOptionValidationError::UnknownOperationMode {
                operation_mode_id: "operation-mode:404".to_string(),
            },
            "invalid_operation_mode",
        ),
        (
            DeviceOptionValidationError::UnknownStopOption {
                stop_option_id: "stop-option:404".to_string(),
            },
            "invalid_stop_option",
        ),
        (
            DeviceOptionValidationError::StopOptionIncompatibleWithMode {
                stop_option_id: "stop-option:1".to_string(),
                operation_mode_id: "operation-mode:202".to_string(),
            },
            "stop_option_incompatible",
        ),
        (
            DeviceOptionValidationError::UnknownChannelMode {
                channel_mode_id: "channel-mode:404".to_string(),
            },
            "invalid_channel_mode",
        ),
        (
            DeviceOptionValidationError::UnsupportedSampleRate {
                sample_rate_hz: 123,
                channel_mode_id: "channel-mode:11".to_string(),
            },
            "sample_rate_unsupported",
        ),
        (
            DeviceOptionValidationError::TooManyEnabledChannels {
                enabled_channel_count: 17,
                max_enabled_channels: 16,
            },
            "enabled_channels_exceed_mode_limit",
        ),
        (
            DeviceOptionValidationError::SampleLimitExceedsCapacity {
                effective_sample_limit: 8192,
                maximum_sample_limit: 4096,
                enabled_channel_count: 4,
            },
            "sample_limit_exceeds_capacity",
        ),
        (
            DeviceOptionValidationError::ThresholdOutOfRange {
                threshold_volts: 6.0,
                min_volts: 0.0,
                max_volts: 5.0,
            },
            "threshold_out_of_range",
        ),
        (
            DeviceOptionValidationError::ThresholdStepInvalid {
                threshold_volts: 1.85,
                min_volts: 0.0,
                step_volts: 0.1,
            },
            "threshold_step_invalid",
        ),
        (
            DeviceOptionValidationError::UnknownFilter {
                filter_id: "filter:404".to_string(),
            },
            "invalid_filter",
        ),
    ];

    for (error, expected_code) in cases {
        assert_eq!(error.code(), expected_code);
    }
}
