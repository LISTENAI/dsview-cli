use std::sync::{Mutex, OnceLock};

use dsview_sys::{source_runtime_library_path, RuntimeBridge};

const SR_OK: i32 = 0;
const SR_ERR_ARG: i32 = 3;
const SR_ERR_NA: i32 = 6;

const SR_CONF_VLD_CH_NUM: i32 = 30027;
const SR_CONF_FILTER: i32 = 30021;
const SR_CONF_OPERATION_MODE: i32 = 30065;
const SR_CONF_BUFFER_OPTIONS: i32 = 30066;
const SR_CONF_CHANNEL_MODE: i32 = 30067;
const SR_CONF_THRESHOLD: i32 = 30071;
const SR_CONF_VTH: i32 = 30072;

const BUFFER_MODE: i32 = 101;
const STREAM_MODE: i32 = 202;

const STOP_WHEN_FULL: i32 = 1;
const UPLOAD_WHEN_FULL: i32 = 2;

const FILTER_NONE: i32 = 0;
const FILTER_ONE_TICK: i32 = 1;

const BUFFER_WIDE_MODE: i32 = 11;
const BUFFER_COMPACT_MODE: i32 = 12;
const STREAM_WIDE_MODE: i32 = 21;
const STREAM_COMPACT_MODE: i32 = 22;

const THRESHOLD_3V3: i32 = 330;
const THRESHOLD_5V0: i32 = 500;

#[repr(C)]
#[derive(Clone, Copy)]
struct TestListItem {
    id: i32,
    name: [u8; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TestChannelMode {
    id: i32,
    name: [u8; 64],
    max_enabled_channels: u16,
}

unsafe extern "C" {
    fn dsview_test_install_mock_option_api();
    fn dsview_test_reset_mock_option_api();
    fn dsview_test_mock_set_current_int(key: i32, has_value: i32, value: i32, status: i32);
    fn dsview_test_mock_set_current_double(key: i32, has_value: i32, value: f64, status: i32);
    fn dsview_test_mock_set_list_items(
        key: i32,
        items: *const TestListItem,
        count: i32,
        status: i32,
    );
    fn dsview_test_mock_set_channel_mode_group(
        operation_mode_code: i32,
        items: *const TestChannelMode,
        count: i32,
        status: i32,
    );
}

fn runtime_test_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn load_runtime() -> Option<RuntimeBridge> {
    let path = source_runtime_library_path()?;
    RuntimeBridge::load(path).ok()
}

fn fixed_name(label: &str) -> [u8; 64] {
    let mut bytes = [0_u8; 64];
    let label_bytes = label.as_bytes();
    let len = label_bytes.len().min(bytes.len().saturating_sub(1));
    bytes[..len].copy_from_slice(&label_bytes[..len]);
    bytes
}

fn list_item(id: i32, name: &str) -> TestListItem {
    TestListItem {
        id,
        name: fixed_name(name),
    }
}

fn channel_mode(id: i32, name: &str, max_enabled_channels: u16) -> TestChannelMode {
    TestChannelMode {
        id,
        name: fixed_name(name),
        max_enabled_channels,
    }
}

fn configure_mock_option_api() {
    unsafe {
        dsview_test_reset_mock_option_api();
        dsview_test_install_mock_option_api();
    }

    let operation_modes = [
        list_item(BUFFER_MODE, "Buffer Mode"),
        list_item(STREAM_MODE, "Stream Mode"),
    ];
    let stop_options = [
        list_item(STOP_WHEN_FULL, "Stop when memory fills"),
        list_item(UPLOAD_WHEN_FULL, "Upload after memory fills"),
    ];
    let filters = [
        list_item(FILTER_NONE, "No filtering"),
        list_item(FILTER_ONE_TICK, "Single-clock filter"),
    ];
    let legacy_thresholds = [
        list_item(THRESHOLD_3V3, "1.8/2.5/3.3V Level"),
        list_item(THRESHOLD_5V0, "5.0V Level"),
    ];
    let buffer_channel_modes = [
        channel_mode(BUFFER_WIDE_MODE, "Buffer wide lanes", 16),
        channel_mode(BUFFER_COMPACT_MODE, "Buffer compact lanes", 8),
    ];
    let stream_channel_modes = [
        channel_mode(STREAM_WIDE_MODE, "Streaming full lanes", 16),
        channel_mode(STREAM_COMPACT_MODE, "Streaming compact lanes", 6),
    ];

    unsafe {
        dsview_test_mock_set_list_items(
            SR_CONF_OPERATION_MODE,
            operation_modes.as_ptr(),
            operation_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_list_items(
            SR_CONF_BUFFER_OPTIONS,
            stop_options.as_ptr(),
            stop_options.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_list_items(
            SR_CONF_FILTER,
            filters.as_ptr(),
            filters.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_list_items(
            SR_CONF_THRESHOLD,
            legacy_thresholds.as_ptr(),
            legacy_thresholds.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_group(
            BUFFER_MODE,
            buffer_channel_modes.as_ptr(),
            buffer_channel_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_group(
            STREAM_MODE,
            stream_channel_modes.as_ptr(),
            stream_channel_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_current_int(SR_CONF_OPERATION_MODE, 1, BUFFER_MODE, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_BUFFER_OPTIONS, 1, STOP_WHEN_FULL, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_FILTER, 1, FILTER_ONE_TICK, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_CHANNEL_MODE, 1, BUFFER_COMPACT_MODE, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_THRESHOLD, 1, THRESHOLD_3V3, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_VLD_CH_NUM, 1, 8, SR_OK);
        dsview_test_mock_set_current_double(SR_CONF_VTH, 1, 1.8, SR_OK);
    }
}

#[test]
fn device_options_snapshot_reads_current_and_supported_values() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    configure_mock_option_api();

    let snapshot = runtime
        .device_options()
        .expect("mocked bridge should return a device option snapshot");

    assert_eq!(snapshot.current_operation_mode_code, Some(BUFFER_MODE as i16));
    assert_eq!(snapshot.current_stop_option_code, Some(STOP_WHEN_FULL as i16));
    assert_eq!(snapshot.current_filter_code, Some(FILTER_ONE_TICK as i16));
    assert_eq!(snapshot.current_channel_mode_code, Some(BUFFER_COMPACT_MODE as i16));

    assert_eq!(snapshot.operation_modes.len(), 2);
    assert_eq!(snapshot.operation_modes[0].code, BUFFER_MODE as i16);
    assert_eq!(snapshot.operation_modes[0].label, "Buffer Mode");
    assert_eq!(snapshot.stop_options[1].code, UPLOAD_WHEN_FULL as i16);
    assert_eq!(snapshot.filters[0].label, "No filtering");

    assert_eq!(snapshot.threshold.kind, "voltage-range");
    assert_eq!(snapshot.threshold.id, "threshold:vth-range");
    assert_eq!(snapshot.threshold.current_volts, Some(1.8));
    assert_eq!(snapshot.threshold.min_volts, 0.0);
    assert_eq!(snapshot.threshold.max_volts, 5.0);
    assert_eq!(snapshot.threshold.step_volts, 0.1);
    assert_eq!(
        snapshot
            .threshold
            .legacy
            .as_ref()
            .expect("legacy threshold metadata should be preserved")
            .current_code,
        Some(THRESHOLD_3V3 as i16)
    );
    assert_eq!(
        snapshot
            .threshold
            .legacy
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["1.8/2.5/3.3V Level", "5.0V Level"]
    );

    let mutated_operation_modes = [list_item(BUFFER_MODE, "Mutated buffer label")];
    unsafe {
        dsview_test_mock_set_list_items(
            SR_CONF_OPERATION_MODE,
            mutated_operation_modes.as_ptr(),
            mutated_operation_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_current_double(SR_CONF_VTH, 1, 3.3, SR_OK);
    }

    assert_eq!(snapshot.operation_modes[0].label, "Buffer Mode");
    assert_eq!(snapshot.threshold.current_volts, Some(1.8));
}

#[test]
fn channel_modes_are_grouped_by_operation_mode() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    configure_mock_option_api();

    let snapshot = runtime
        .device_options()
        .expect("mocked bridge should return grouped channel modes");

    assert_eq!(snapshot.channel_mode_groups.len(), 2);
    assert_eq!(
        snapshot
            .channel_mode_groups
            .iter()
            .map(|group| group.operation_mode_code)
            .collect::<Vec<_>>(),
        vec![BUFFER_MODE as i16, STREAM_MODE as i16]
    );
    assert_eq!(
        snapshot.channel_mode_groups[0]
            .channel_modes
            .iter()
            .map(|mode| (mode.code, mode.label.as_str(), mode.max_enabled_channels))
            .collect::<Vec<_>>(),
        vec![
            (BUFFER_WIDE_MODE as i16, "Buffer wide lanes", 16),
            (BUFFER_COMPACT_MODE as i16, "Buffer compact lanes", 8),
        ]
    );
    assert_eq!(
        snapshot.channel_mode_groups[1]
            .channel_modes
            .iter()
            .map(|mode| (mode.code, mode.label.as_str(), mode.max_enabled_channels))
            .collect::<Vec<_>>(),
        vec![
            (STREAM_WIDE_MODE as i16, "Streaming full lanes", 16),
            (STREAM_COMPACT_MODE as i16, "Streaming compact lanes", 6),
        ]
    );
}

