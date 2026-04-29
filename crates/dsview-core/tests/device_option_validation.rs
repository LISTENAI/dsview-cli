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
        operation_modes: vec![
            OperationModeValidationCapabilities {
                id: "operation-mode:101".to_string(),
                native_code: 101,
                label: "Buffer Mode".to_string(),
                stop_option_ids: vec!["stop-option:1".to_string(), "stop-option:2".to_string()],
                channel_modes: vec![
                    ChannelModeValidationCapabilities {
                        id: "channel-mode:11".to_string(),
                        native_code: 11,
                        label: "Buffer 100x16".to_string(),
                        max_enabled_channels: 16,
                        supported_sample_rates: vec![50_000_000, 100_000_000],
                    },
                    ChannelModeValidationCapabilities {
                        id: "channel-mode:13".to_string(),
                        native_code: 13,
                        label: "Buffer 400x4".to_string(),
                        max_enabled_channels: 4,
                        supported_sample_rates: vec![100_000_000, 200_000_000, 400_000_000],
                    },
                ],
            },
            OperationModeValidationCapabilities {
                id: "operation-mode:202".to_string(),
                native_code: 202,
                label: "Stream Mode".to_string(),
                stop_option_ids: Vec::new(),
                channel_modes: vec![ChannelModeValidationCapabilities {
                    id: "channel-mode:21".to_string(),
                    native_code: 21,
                    label: "Stream 20x16".to_string(),
                    max_enabled_channels: 16,
                    supported_sample_rates: vec![20_000_000, 25_000_000],
                }],
            },
        ],
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
            "operation_mode_unsupported",
        ),
        (
            DeviceOptionValidationError::UnknownStopOption {
                stop_option_id: "stop-option:404".to_string(),
            },
            "stop_option_unsupported",
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
            "channel_mode_unsupported",
        ),
        (
            DeviceOptionValidationError::ChannelModeIncompatibleWithOperationMode {
                operation_mode_id: "operation-mode:202".to_string(),
                channel_mode_id: "channel-mode:11".to_string(),
            },
            "channel_mode_incompatible",
        ),
        (
            DeviceOptionValidationError::EmptySampleRate,
            "sample_rate_missing",
        ),
        (
            DeviceOptionValidationError::EmptySampleLimit,
            "sample_limit_missing",
        ),
        (
            DeviceOptionValidationError::NoEnabledChannels,
            "enabled_channels_empty",
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
            "filter_unsupported",
        ),
    ];

    for (error, expected_code) in cases {
        assert_eq!(error.code(), expected_code);
    }
}

#[test]
fn buffer_100x16_accepts_100mhz_and_four_channels() {
    let capabilities = validation_capabilities();
    let request = validation_request();

    let validated = capabilities.validate_request(&request).unwrap();

    assert_eq!(validated.operation_mode_id, "operation-mode:101");
    assert_eq!(validated.operation_mode_code, 101);
    assert_eq!(validated.channel_mode_id, "channel-mode:11");
    assert_eq!(validated.channel_mode_code, 11);
    assert_eq!(validated.sample_rate_hz, 100_000_000);
    assert_eq!(validated.requested_sample_limit, 4096);
    assert_eq!(validated.effective_sample_limit, 4096);
    assert_eq!(validated.enabled_channels, vec![0, 1, 2, 3]);
    assert_eq!(validated.stop_option_code, Some(1));
    assert_eq!(validated.filter_code, Some(1));
}

#[test]
fn buffer_400x4_rejects_five_enabled_channels() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        channel_mode_id: "channel-mode:13".to_string(),
        enabled_channels: [0_u16, 1, 2, 3, 4].into_iter().collect(),
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::TooManyEnabledChannels {
            enabled_channel_count: 5,
            max_enabled_channels: 4,
        }
    );
    assert_eq!(error.code(), "enabled_channels_exceed_mode_limit");
}

#[test]
fn buffer_100x16_rejects_200mhz_samplerate() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        sample_rate_hz: 200_000_000,
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::UnsupportedSampleRate {
            sample_rate_hz: 200_000_000,
            channel_mode_id: "channel-mode:11".to_string(),
        }
    );
    assert_eq!(error.code(), "sample_rate_unsupported");
}

#[test]
fn sample_limit_alignment_can_push_request_over_capacity() {
    let mut capabilities = validation_capabilities();
    capabilities.hardware_sample_capacity = 14_336;
    let request = DeviceOptionValidationRequest {
        sample_limit: 3_073,
        enabled_channels: [0_u16, 1, 2, 3].into_iter().collect(),
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::SampleLimitExceedsCapacity {
            effective_sample_limit: 4_096,
            maximum_sample_limit: 3_072,
            enabled_channel_count: 4,
        }
    );
}

#[test]
fn stream_mode_accepts_sample_limit_beyond_buffer_capacity() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        operation_mode_id: "operation-mode:202".to_string(),
        channel_mode_id: "channel-mode:21".to_string(),
        stop_option_id: None,
        sample_rate_hz: 25_000_000,
        sample_limit: 300_000_000,
        ..validation_request()
    };

    let validated = capabilities.validate_request(&request).unwrap();

    assert_eq!(validated.operation_mode_id, "operation-mode:202");
    assert_eq!(validated.channel_mode_id, "channel-mode:21");
    assert_eq!(validated.requested_sample_limit, 300_000_000);
    assert_eq!(validated.effective_sample_limit, 300_000_256);
}

#[test]
fn threshold_value_must_stay_within_vth_range() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        threshold_volts: Some(5.1),
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::ThresholdOutOfRange {
            threshold_volts: 5.1,
            min_volts: 0.0,
            max_volts: 5.0,
        }
    );
    assert_eq!(error.code(), "threshold_out_of_range");
}

#[test]
fn threshold_value_must_follow_point_one_volt_steps() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        threshold_volts: Some(1.85),
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::ThresholdStepInvalid {
            threshold_volts: 1.85,
            min_volts: 0.0,
            step_volts: 0.1,
        }
    );
    assert_eq!(error.code(), "threshold_step_invalid");
}

#[test]
fn threshold_value_must_reject_nan() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        threshold_volts: Some(f64::NAN),
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert!(matches!(
        error,
        DeviceOptionValidationError::ThresholdOutOfRange {
            threshold_volts,
            min_volts: 0.0,
            max_volts: 5.0,
        } if threshold_volts.is_nan()
    ));
    assert_eq!(error.code(), "threshold_out_of_range");
}

#[test]
fn filter_id_must_exist_in_supported_list() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        filter_id: Some("filter:99".to_string()),
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::UnknownFilter {
            filter_id: "filter:99".to_string(),
        }
    );
    assert_eq!(error.code(), "filter_unsupported");
}

#[test]
fn stream_mode_rejects_buffer_only_stop_option() {
    let capabilities = validation_capabilities();
    let request = DeviceOptionValidationRequest {
        operation_mode_id: "operation-mode:202".to_string(),
        channel_mode_id: "channel-mode:21".to_string(),
        stop_option_id: Some("stop-option:1".to_string()),
        sample_rate_hz: 25_000_000,
        ..validation_request()
    };

    let error = capabilities.validate_request(&request).unwrap_err();

    assert_eq!(
        error,
        DeviceOptionValidationError::StopOptionIncompatibleWithMode {
            stop_option_id: "stop-option:1".to_string(),
            operation_mode_id: "operation-mode:202".to_string(),
        }
    );
    assert_eq!(error.code(), "stop_option_incompatible");
}
