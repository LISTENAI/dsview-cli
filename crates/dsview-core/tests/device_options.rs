use dsview_core::{
    normalize_device_options_snapshot, DeviceHandle, SelectionHandle, SupportedDevice,
    SupportedDeviceKind,
};
use dsview_sys::{
    DeviceOptionChannelMode, DeviceOptionChannelModeGroup, DeviceOptionValue,
    DeviceOptionsSnapshot as NativeDeviceOptionsSnapshot, ThresholdVoltageRange,
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

    assert_eq!(snapshot.current.operation_mode_id.as_deref(), Some("operation-mode:2"));
    assert_eq!(snapshot.current.stop_option_id.as_deref(), Some("stop-option:9"));
    assert_eq!(snapshot.current.filter_id.as_deref(), Some("filter:4"));
    assert_eq!(snapshot.current.channel_mode_id.as_deref(), Some("channel-mode:22"));
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
        snapshot.channel_modes_by_operation_mode[1].current_channel_mode_id.as_deref(),
        Some("channel-mode:22")
    );
    assert_eq!(
        snapshot.channel_modes_by_operation_mode[1].current_channel_mode_code,
        Some(22)
    );
}
