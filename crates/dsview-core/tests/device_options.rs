use dsview_core::{
    DecoderDescriptor, DeviceHandle, SelectionHandle, SupportedDevice, SupportedDeviceKind,
    normalize_decoder_descriptor, normalize_device_options_snapshot,
};
use dsview_sys::{
    DecodeAnnotation, DecodeAnnotationRow, DecodeChannel, DecodeDecoder, DecodeInput, DecodeOption,
    DecodeOptionValueKind, DecodeOutput, DeviceOptionChannelMode, DeviceOptionChannelModeGroup,
    DeviceOptionValue, DeviceOptionsSnapshot as NativeDeviceOptionsSnapshot,
    LegacyThresholdMetadata as NativeLegacyThresholdMetadata, ThresholdVoltageRange,
};

fn supported_device() -> SupportedDevice {
    SupportedDevice {
        selection_handle: SelectionHandle::new(1).unwrap(),
        native_handle: DeviceHandle::new(7).unwrap(),
        name: "DSLogic PLus".to_string(),
        kind: SupportedDeviceKind::DsLogicPlus,
        stable_id: SupportedDeviceKind::DsLogicPlus.stable_id(),
    }
}

fn native_snapshot() -> NativeDeviceOptionsSnapshot {
    NativeDeviceOptionsSnapshot {
        current_operation_mode_code: Some(2),
        operation_modes: vec![
            DeviceOptionValue {
                code: 2,
                label: "Stream Mode".to_string(),
            },
            DeviceOptionValue {
                code: 1,
                label: "Buffer Mode".to_string(),
            },
        ],
        current_stop_option_code: Some(9),
        stop_options: vec![
            DeviceOptionValue {
                code: 9,
                label: "Upload Captured Data".to_string(),
            },
            DeviceOptionValue {
                code: 5,
                label: "Stop Right Away".to_string(),
            },
        ],
        current_filter_code: Some(4),
        filters: vec![
            DeviceOptionValue {
                code: 4,
                label: "1 Sample Clock".to_string(),
            },
            DeviceOptionValue {
                code: 0,
                label: "No Filter".to_string(),
            },
        ],
        current_channel_mode_code: Some(22),
        channel_mode_groups: vec![
            DeviceOptionChannelModeGroup {
                operation_mode_code: 2,
                channel_modes: vec![
                    DeviceOptionChannelMode {
                        code: 22,
                        label: "Use Channels 0~7 (Max 200MHz)".to_string(),
                        max_enabled_channels: 8,
                    },
                    DeviceOptionChannelMode {
                        code: 21,
                        label: "Use Channels 0~15 (Max 100MHz)".to_string(),
                        max_enabled_channels: 16,
                    },
                ],
            },
            DeviceOptionChannelModeGroup {
                operation_mode_code: 1,
                channel_modes: vec![
                    DeviceOptionChannelMode {
                        code: 12,
                        label: "Buffer 8".to_string(),
                        max_enabled_channels: 8,
                    },
                    DeviceOptionChannelMode {
                        code: 11,
                        label: "Buffer 16".to_string(),
                        max_enabled_channels: 16,
                    },
                ],
            },
        ],
        threshold: ThresholdVoltageRange {
            kind: "float-range".to_string(),
            id: "native-threshold".to_string(),
            current_volts: Some(1.8),
            min_volts: 0.7,
            max_volts: 4.0,
            step_volts: 0.1,
            legacy: None,
        },
    }
}

fn native_snapshot_with_legacy_threshold() -> NativeDeviceOptionsSnapshot {
    let mut snapshot = native_snapshot();
    snapshot.threshold.legacy = Some(NativeLegacyThresholdMetadata {
        current_code: Some(3),
        options: vec![
            DeviceOptionValue {
                code: 7,
                label: "TTL".to_string(),
            },
            DeviceOptionValue {
                code: 3,
                label: "CMOS".to_string(),
            },
        ],
    });
    snapshot
}

fn native_decoder() -> DecodeDecoder {
    DecodeDecoder {
        id: "0:i2c".to_string(),
        name: "i2c".to_string(),
        longname: "Inter-Integrated Circuit".to_string(),
        description: "serial bus decoder".to_string(),
        license: "gplv2+".to_string(),
        inputs: vec![DecodeInput {
            id: "logic".to_string(),
        }],
        outputs: vec![
            DecodeOutput {
                id: "i2c".to_string(),
            },
            DecodeOutput {
                id: "i2c-messages".to_string(),
            },
        ],
        tags: vec!["serial".to_string(), "embedded".to_string()],
        required_channels: vec![DecodeChannel {
            id: "scl".to_string(),
            name: "SCL".to_string(),
            description: "clock".to_string(),
            order: 0,
            channel_type: 0,
            idn: Some("clk".to_string()),
        }],
        optional_channels: vec![DecodeChannel {
            id: "sda".to_string(),
            name: "SDA".to_string(),
            description: "data".to_string(),
            order: 1,
            channel_type: 0,
            idn: Some("data".to_string()),
        }],
        options: vec![DecodeOption {
            id: "address_format".to_string(),
            idn: Some("address_format".to_string()),
            description: Some("show 7-bit or 8-bit addresses".to_string()),
            value_kind: DecodeOptionValueKind::String,
            default_value: Some("7-bit".to_string()),
            values: vec!["7-bit".to_string(), "8-bit".to_string()],
        }],
        annotations: vec![DecodeAnnotation {
            id: "start".to_string(),
            label: Some("START".to_string()),
            description: Some("start condition".to_string()),
            annotation_type: 0,
        }],
        annotation_rows: vec![DecodeAnnotationRow {
            id: "frames".to_string(),
            description: Some("frame events".to_string()),
            annotation_classes: vec![0],
        }],
    }
}

#[test]
fn normalizes_option_ids_without_label_parsing() {
    // Contract: operation-mode:<code>, stop-option:<code>, filter:<code>, channel-mode:<code>.
    let snapshot = normalize_device_options_snapshot(&supported_device(), native_snapshot());

    assert_eq!(
        snapshot
            .operation_modes
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec!["operation-mode:1", "operation-mode:2"]
    );
    assert_eq!(
        snapshot
            .stop_options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec!["stop-option:5", "stop-option:9"]
    );
    assert_eq!(
        snapshot
            .filters
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        vec!["filter:0", "filter:4"]
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[0]
            .channel_modes
            .iter()
            .map(|mode| mode.id.as_str())
            .collect::<Vec<_>>(),
        vec!["channel-mode:11", "channel-mode:12"]
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[1]
            .channel_modes
            .iter()
            .map(|mode| mode.id.as_str())
            .collect::<Vec<_>>(),
        vec!["channel-mode:21", "channel-mode:22"]
    );

    assert_eq!(
        snapshot.current.operation_mode_id.as_deref(),
        Some("operation-mode:2")
    );
    assert_eq!(
        snapshot.current.stop_option_id.as_deref(),
        Some("stop-option:9")
    );
    assert_eq!(snapshot.current.filter_id.as_deref(), Some("filter:4"));
    assert_eq!(
        snapshot.current.channel_mode_id.as_deref(),
        Some("channel-mode:22")
    );
}

#[test]
fn selected_device_snapshot_preserves_channel_mode_groups() {
    let snapshot = normalize_device_options_snapshot(&supported_device(), native_snapshot());

    assert_eq!(snapshot.device.selection_handle, 1);
    assert_eq!(snapshot.device.native_handle, 7);
    assert_eq!(snapshot.device.stable_id, "dslogic-plus");
    assert_eq!(
        snapshot
            .channel_modes_by_operation_mode
            .iter()
            .map(|group| group.operation_mode_id.as_str())
            .collect::<Vec<_>>(),
        vec!["operation-mode:1", "operation-mode:2"]
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[0]
            .channel_modes
            .iter()
            .map(|mode| mode.native_code)
            .collect::<Vec<_>>(),
        vec![11, 12]
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[0].current_channel_mode_id,
        None
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[1]
            .current_channel_mode_id
            .as_deref(),
        Some("channel-mode:22")
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[1].current_channel_mode_code,
        Some(22)
    );
}

#[test]
fn threshold_snapshot_uses_voltage_range_contract_and_keeps_legacy_metadata_raw() {
    let snapshot = normalize_device_options_snapshot(
        &supported_device(),
        native_snapshot_with_legacy_threshold(),
    );

    assert_eq!(snapshot.threshold.id, "threshold:vth-range");
    assert_eq!(snapshot.threshold.kind, "voltage-range");
    assert_eq!(snapshot.threshold.current_volts, Some(1.8));
    assert_eq!(snapshot.threshold.min_volts, 0.7);
    assert_eq!(snapshot.threshold.max_volts, 4.0);
    assert_eq!(snapshot.threshold.step_volts, 0.1);
    assert_eq!(
        snapshot
            .threshold
            .legacy_metadata
            .as_ref()
            .unwrap()
            .current_native_code,
        Some(3)
    );
    assert_eq!(
        snapshot
            .threshold
            .legacy_metadata
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.native_code)
            .collect::<Vec<_>>(),
        vec![3, 7]
    );
}

#[test]
fn preserves_upstream_decoder_ids() {
    let descriptor: DecoderDescriptor = normalize_decoder_descriptor(native_decoder());

    assert_eq!(descriptor.id, "0:i2c");
    assert_eq!(descriptor.required_channels[0].id, "scl");
    assert_eq!(descriptor.optional_channels[0].id, "sda");
    assert_eq!(descriptor.options[0].id, "address_format");
    assert_eq!(descriptor.annotation_rows[0].id, "frames");
}

#[test]
fn keeps_required_and_optional_channels_distinct() {
    let descriptor = normalize_decoder_descriptor(native_decoder());

    assert_eq!(
        descriptor
            .required_channels
            .iter()
            .map(|channel| channel.id.as_str())
            .collect::<Vec<_>>(),
        vec!["scl"]
    );
    assert_eq!(
        descriptor
            .optional_channels
            .iter()
            .map(|channel| channel.id.as_str())
            .collect::<Vec<_>>(),
        vec!["sda"]
    );
}

#[test]
fn exposes_stack_relevant_inputs_and_outputs() {
    let descriptor = normalize_decoder_descriptor(native_decoder());

    assert_eq!(
        descriptor
            .inputs
            .iter()
            .map(|input| input.id.as_str())
            .collect::<Vec<_>>(),
        vec!["logic"]
    );
    assert_eq!(
        descriptor
            .outputs
            .iter()
            .map(|output| output.id.as_str())
            .collect::<Vec<_>>(),
        vec!["i2c", "i2c-messages"]
    );
    assert_eq!(descriptor.annotation_rows[0].annotation_classes, vec![0]);
}

#[test]
fn normalizes_python_repr_decoder_option_values_for_config_files() {
    let mut native = native_decoder();
    native.options = vec![
        DecodeOption {
            id: "invert".to_string(),
            idn: Some("invert".to_string()),
            description: Some("Invert signal".to_string()),
            value_kind: DecodeOptionValueKind::String,
            default_value: Some("'no'".to_string()),
            values: vec!["'yes'".to_string(), "'no'".to_string()],
        },
        DecodeOption {
            id: "num_stop_bits".to_string(),
            idn: Some("num_stop_bits".to_string()),
            description: Some("Stop bits".to_string()),
            value_kind: DecodeOptionValueKind::Float,
            default_value: Some("1.0".to_string()),
            values: vec!["0.0".to_string(), "1.0".to_string(), "1.5".to_string()],
        },
        DecodeOption {
            id: "num_data_bits".to_string(),
            idn: Some("num_data_bits".to_string()),
            description: Some("Data bits".to_string()),
            value_kind: DecodeOptionValueKind::Unknown,
            default_value: Some("int64 8".to_string()),
            values: vec!["int64 7".to_string(), "int64 8".to_string()],
        },
    ];

    let descriptor = normalize_decoder_descriptor(native);

    assert_eq!(descriptor.options[0].default_value.as_deref(), Some("no"));
    assert_eq!(descriptor.options[0].values, vec!["yes", "no"]);
    assert_eq!(descriptor.options[1].default_value.as_deref(), Some("1"));
    assert_eq!(descriptor.options[1].values, vec!["0", "1", "1.5"]);
    assert_eq!(descriptor.options[2].default_value.as_deref(), Some("8"));
    assert_eq!(descriptor.options[2].values, vec!["7", "8"]);
    assert_eq!(
        descriptor.options[2].value_kind,
        DecodeOptionValueKind::Integer
    );
}
