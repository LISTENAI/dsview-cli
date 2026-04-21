use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use dsview_sys::{
    decode_runtime_library_name, runtime_library_name, session_send_logic_chunk,
    source_decode_runtime_library_path, source_runtime_library_path, upstream_header_path,
    DecodeDecoder, DecodeExecutionLogicFormat, DecodeExecutionSession, DecodeRuntimeBridge,
    DecodeOptionValueKind, DecodeRuntimeError, DecodeRuntimeErrorCode, DecodeSessionChannelBinding,
    DecodeSessionInstance, DecodeSessionOption, DecodeSessionOptionValue, ExportErrorCode,
    RuntimeBridge, RuntimeError, VcdExportRequest,
};

fn runtime_test_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn load_runtime() -> Option<RuntimeBridge> {
    let path = source_runtime_library_path()?;
    RuntimeBridge::load(path).ok()
}

fn skip_windows_vcd_goldens() -> bool {
    cfg!(target_os = "windows")
}

fn load_decode_runtime() -> Option<DecodeRuntimeBridge> {
    let path = source_decode_runtime_library_path()?;
    DecodeRuntimeBridge::load(path).ok()
}

fn i2c_root_instance() -> DecodeSessionInstance {
    DecodeSessionInstance {
        decoder_id: "0:i2c".to_string(),
        channel_bindings: vec![
            DecodeSessionChannelBinding {
                channel_id: "scl".to_string(),
                channel_index: 0,
            },
            DecodeSessionChannelBinding {
                channel_id: "sda".to_string(),
                channel_index: 1,
            },
        ],
        options: vec![],
    }
}

fn eeprom_stack_instance() -> DecodeSessionInstance {
    DecodeSessionInstance {
        decoder_id: "eeprom24xx".to_string(),
        channel_bindings: vec![],
        options: vec![DecodeSessionOption {
            option_id: "addr_counter".to_string(),
            value: DecodeSessionOptionValue::Integer(0),
        }],
    }
}

fn i2c_logic_sample(scl: bool, sda: bool) -> u8 {
    (u8::from(scl)) | (u8::from(sda) << 1)
}

fn push_i2c_state(samples: &mut Vec<u8>, scl: bool, sda: bool) {
    samples.push(i2c_logic_sample(scl, sda));
}

fn append_i2c_start(samples: &mut Vec<u8>) {
    push_i2c_state(samples, true, true);
    push_i2c_state(samples, true, false);
    push_i2c_state(samples, false, false);
}

fn append_i2c_bit(samples: &mut Vec<u8>, bit: bool) {
    push_i2c_state(samples, false, bit);
    push_i2c_state(samples, true, bit);
    push_i2c_state(samples, true, bit);
    push_i2c_state(samples, false, bit);
}

fn append_i2c_ack(samples: &mut Vec<u8>, ack: bool) {
    append_i2c_bit(samples, !ack);
}

fn append_i2c_byte(samples: &mut Vec<u8>, byte: u8) {
    for bit_index in (0..8).rev() {
        append_i2c_bit(samples, ((byte >> bit_index) & 1) != 0);
    }
}

fn append_i2c_stop(samples: &mut Vec<u8>) {
    push_i2c_state(samples, false, false);
    push_i2c_state(samples, true, false);
    push_i2c_state(samples, true, true);
    push_i2c_state(samples, true, true);
}

fn eeprom_byte_write_trace() -> Vec<u8> {
    let mut samples = Vec::new();
    append_i2c_start(&mut samples);
    append_i2c_byte(&mut samples, 0xA0);
    append_i2c_ack(&mut samples, true);
    append_i2c_byte(&mut samples, 0x00);
    append_i2c_ack(&mut samples, true);
    append_i2c_byte(&mut samples, 0xAB);
    append_i2c_ack(&mut samples, true);
    append_i2c_stop(&mut samples);
    samples
}

fn source_decoder_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .join("DSView")
        .join("libsigrokdecode4DSL")
        .join("decoders")
}

fn vcd_string(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("synthetic VCD should be utf-8")
}

fn normalize_vcd(vcd: &str) -> String {
    let mut normalized = Vec::new();
    let mut lines = vcd.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("$date") {
            normalized.push("$date <normalized> $end".to_string());
            continue;
        }
        if trimmed.starts_with("$version") {
            normalized.push("$version <normalized> $end".to_string());
            continue;
        }
        if trimmed == "$comment" {
            normalized.push("$comment".to_string());
            normalized.push("<normalized>".to_string());
            for comment_line in lines.by_ref() {
                if comment_line.trim() == "$end" {
                    normalized.push("$end".to_string());
                    break;
                }
            }
            continue;
        }
        normalized.push(trimmed.to_string());
    }

    normalized.join("\n") + "\n"
}

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("goldens")
        .join(name)
}

fn temp_path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dsview-sys-{name}-{nonce}.vcd"))
}

fn parse_transitions(vcd: &str) -> Vec<(u64, String)> {
    let mut now = 0_u64;
    let mut transitions = Vec::new();

    for line in vcd.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(rest) = line.strip_prefix('#') {
            let mut parts = rest.split_whitespace();
            let timestamp = parts.next().expect("timestamp should exist");
            now = timestamp.parse().expect("timestamp should parse");
            for value in parts {
                transitions.push((now, value.to_string()));
            }
            continue;
        }
        if matches!(line, "$dumpvars" | "$end") || line.starts_with('$') {
            continue;
        }
        if matches!(line.as_bytes().first(), Some(b'0' | b'1' | b'x' | b'z')) {
            transitions.push((now, line.to_string()));
        }
    }

    transitions
}

fn final_timestamp(vcd: &str) -> Option<u64> {
    vcd.lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('#'))
        .filter_map(|raw| raw.split_whitespace().next())
        .filter_map(|raw| raw.parse::<u64>().ok())
        .last()
}

fn expand_cross_logic_blocks(blocks: &[&[u64]]) -> Vec<u8> {
    let channel_count = blocks.len();
    assert!(channel_count > 0);
    let block_count = blocks[0].len();
    assert!(blocks.iter().all(|channel| channel.len() == block_count));

    let mut expanded = vec![0_u8; block_count * 64];
    for (channel_index, channel_blocks) in blocks.iter().enumerate() {
        for (block_index, block) in channel_blocks.iter().enumerate() {
            for bit_index in 0..64 {
                if ((block >> bit_index) & 1) != 0 {
                    expanded[block_index * 64 + bit_index] |= 1 << channel_index;
                }
            }
        }
    }

    expanded
}

fn pack_cross_logic_blocks(blocks: &[&[u64]]) -> Vec<u8> {
    let channel_count = blocks.len();
    assert!(channel_count > 0);
    let block_count = blocks[0].len();
    assert!(blocks.iter().all(|channel| channel.len() == block_count));

    let mut packed = Vec::with_capacity(block_count * channel_count * std::mem::size_of::<u64>());
    for block_index in 0..block_count {
        for channel_blocks in blocks {
            packed.extend_from_slice(&channel_blocks[block_index].to_le_bytes());
        }
    }

    packed
}

fn with_decode_execution_session(
    assertion: impl FnOnce(&mut DecodeExecutionSession),
) {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_decode_runtime() else {
        return;
    };
    let decoder_dir = source_decoder_dir();
    if !decoder_dir.exists() {
        return;
    }

    runtime
        .init(&decoder_dir)
        .expect("decode runtime should initialize with source decoder dir");

    let mut session =
        DecodeExecutionSession::new().expect("decode execution session should construct");
    session
        .set_samplerate_hz(1_000_000)
        .expect("decode execution session should accept samplerate");
    session
        .build_linear_stack(&i2c_root_instance(), &[])
        .expect("decode execution session should build a linear root stack");
    session
        .start()
        .expect("decode execution session should start");

    assertion(&mut session);

    drop(session);
    runtime.exit().expect("decode runtime exit should succeed");
}

#[test]
fn upstream_header_exists_for_boundary_tests() {
    let header = upstream_header_path();
    assert!(header.ends_with("DSView/libsigrok4DSL/libsigrok.h"));
    assert!(header.exists(), "expected upstream header at {}", header.display());
}

#[test]
fn source_runtime_path_shape_matches_cfg_state() {
    if let Some(path) = source_runtime_library_path() {
        assert!(Path::new(path).is_absolute());
        assert!(path.to_string_lossy().ends_with(runtime_library_name()));
    }
}

#[test]
fn source_decode_runtime_path_shape_matches_cfg_state() {
    if let Some(path) = source_decode_runtime_library_path() {
        assert!(Path::new(path).is_absolute());
        assert!(path.to_string_lossy().ends_with(decode_runtime_library_name()));
    }
}

#[test]
fn decode_runtime_reports_loader_failures() {
    let missing = std::env::temp_dir().join(format!(
        "missing-{}-{}",
        decode_runtime_library_name(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let error = DecodeRuntimeBridge::load(&missing).unwrap_err();

    assert!(matches!(
        error,
        DecodeRuntimeError::LibraryLoad { path, .. } if path == missing
    ));
}

#[test]
fn decode_runtime_init_shapes_decoder_directory_errors() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_decode_runtime() else {
        return;
    };
    let missing = std::env::temp_dir().join(format!(
        "missing-decoders-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let error = runtime.init(&missing).unwrap_err();

    assert!(matches!(
        error,
        DecodeRuntimeError::NativeCall {
            operation: "decode runtime init",
            code: DecodeRuntimeErrorCode::DecoderDirectory,
            ..
        }
    ));
}

#[test]
fn decode_runtime_lists_and_inspects_decoder_metadata() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_decode_runtime() else {
        return;
    };
    let decoder_dir = source_decoder_dir();
    if !decoder_dir.exists() {
        return;
    }

    runtime
        .init(&decoder_dir)
        .expect("decode runtime should initialize with source decoder dir");
    let list = runtime.decode_list().expect("decode_list should succeed");
    let list_entry = list
        .iter()
        .find(|decoder| decoder.id == "0:i2c")
        .expect("expected canonical upstream decoder ids in decode runtime list");

    let decoder = runtime
        .decode_inspect("0:i2c")
        .expect("decode_inspect should return decoder metadata");
    assert_eq!(decoder.id, "0:i2c");
    assert_eq!(decoder.id, list_entry.id, "preserve upstream id across list/inspect");
    assert_eq!(
        decoder.longname, list_entry.longname,
        "decode_inspect should preserve upstream labels from decode_list"
    );
    assert!(!decoder.required_channels.is_empty());
    assert!(!decoder.annotation_rows.is_empty());
    assert!(!decoder.inputs.is_empty());
    assert!(!decoder.outputs.is_empty());

    let unknown = runtime.decode_inspect("missing-decoder").unwrap_err();
    assert!(matches!(
        unknown,
        DecodeRuntimeError::NativeCall {
            operation: "decode inspect",
            code: DecodeRuntimeErrorCode::UnknownDecoder,
            ..
        }
    ));

    runtime.exit().expect("decode runtime exit should succeed");
}

#[test]
fn decode_option_value_kind_follows_upstream_default_type() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_decode_runtime() else {
        return;
    };
    let decoder_dir = source_decoder_dir();
    if !decoder_dir.exists() {
        return;
    }

    runtime
        .init(&decoder_dir)
        .expect("decode runtime should initialize with source decoder dir");

    let list = runtime.decode_list().expect("decode_list should succeed");

    let i2c = runtime
        .decode_inspect("0:i2c")
        .expect("i2c metadata should load");
    let address_format = i2c
        .options
        .iter()
        .find(|option| option.id == "address_format")
        .expect("i2c should expose address_format");
    assert_eq!(address_format.id, "address_format");
    assert_eq!(address_format.value_kind, DecodeOptionValueKind::String);

    let integer_decoder_id = list
        .iter()
        .find_map(|decoder| {
            let inspected = runtime.decode_inspect(&decoder.id).ok()?;
            inspected
                .options
                .iter()
                .any(|option| option.id == "coilfreq")
                .then_some(inspected.id)
        })
        .expect("expected at least one integer option fixture");
    let integer_decoder = runtime
        .decode_inspect(&integer_decoder_id)
        .expect("integer fixture metadata should load");
    let integer_option = integer_decoder
        .options
        .iter()
        .find(|option| option.id == "coilfreq")
        .expect("integer fixture should expose coilfreq");
    assert_eq!(integer_option.id, "coilfreq");
    assert_eq!(integer_option.value_kind, DecodeOptionValueKind::Integer);

    let float_decoder_id = list
        .iter()
        .find_map(|decoder| {
            let inspected = runtime.decode_inspect(&decoder.id).ok()?;
            inspected
                .options
                .iter()
                .any(|option| option.id == "sample_point")
                .then_some(inspected.id)
        })
        .expect("expected at least one float option fixture");
    let float_decoder = runtime
        .decode_inspect(&float_decoder_id)
        .expect("float fixture metadata should load");
    let float_option = float_decoder
        .options
        .iter()
        .find(|option| option.id == "sample_point")
        .expect("float fixture should expose sample_point");
    assert_eq!(float_option.id, "sample_point");
    assert_eq!(float_option.value_kind, DecodeOptionValueKind::Float);

    runtime.exit().expect("decode runtime exit should succeed");
}

#[test]
fn safe_decode_decoder_wrapper_preserves_fixture_ids_and_labels() {
    let decoder = DecodeDecoder {
        id: "0:i2c".to_string(),
        name: "i2c".to_string(),
        longname: "Inter-Integrated Circuit".to_string(),
        description: "fixture decoder".to_string(),
        license: "gplv2+".to_string(),
        inputs: vec![],
        outputs: vec![],
        tags: vec!["serial".to_string()],
        required_channels: vec![],
        optional_channels: vec![],
        options: vec![],
        annotations: vec![],
        annotation_rows: vec![],
    };

    assert_eq!(decoder.id, "0:i2c", "preserve upstream id in safe wrapper");
    assert_eq!(
        decoder.longname, "Inter-Integrated Circuit",
        "preserve upstream label in safe wrapper"
    );
}

#[test]
fn offline_decode_rejects_empty_sample_bytes() {
    with_decode_execution_session(|session| {
        let error = session_send_logic_chunk(
            session,
            0,
            &[],
            DecodeExecutionLogicFormat::SplitLogic { unitsize: 1 },
            None,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            DecodeRuntimeError::InvalidArgument(detail) if detail.contains("sample bytes must not be empty")
        ));
    });
}

#[test]
fn offline_decode_rejects_misaligned_logic_packet_lengths() {
    with_decode_execution_session(|session| {
        let sample_bytes = [0b00, 0b01, 0b10];
        let packet_lengths = [2_usize, 1_usize];

        let error = session_send_logic_chunk(
            session,
            0,
            &sample_bytes,
            DecodeExecutionLogicFormat::SplitLogic { unitsize: 2 },
            Some(&packet_lengths),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            DecodeRuntimeError::InvalidArgument(detail)
                if detail.contains("aligned to the requested logic format")
        ));
    });
}

#[test]
fn offline_decode_session_wrappers_require_absolute_sample_progression() {
    with_decode_execution_session(|session| {
        let first = [0b00, 0b01];
        session_send_logic_chunk(
            session,
            0,
            &first,
            DecodeExecutionLogicFormat::SplitLogic { unitsize: 1 },
            None,
        )
        .expect("first chunk should establish the absolute cursor");

        let second = [0b10, 0b11];
        let error = session_send_logic_chunk(
            session,
            0,
            &second,
            DecodeExecutionLogicFormat::SplitLogic { unitsize: 1 },
            None,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            DecodeRuntimeError::InvalidArgument(detail)
                if detail.contains("absolute sample progression")
        ));
    });
}

#[test]
fn stacked_decoder_python_output_flows_linearly() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_decode_runtime() else {
        return;
    };
    let decoder_dir = source_decoder_dir();
    if !decoder_dir.exists() {
        return;
    }

    runtime
        .init(&decoder_dir)
        .expect("decode runtime should initialize with source decoder dir");

    let mut session =
        DecodeExecutionSession::new().expect("decode execution session should construct");
    session
        .set_samplerate_hz(1_000_000)
        .expect("decode execution session should accept samplerate");
    session
        .build_linear_stack(&i2c_root_instance(), &[eeprom_stack_instance()])
        .expect("decode execution session should build a stacked EEPROM chain");
    session
        .start()
        .expect("decode execution session should start");

    let samples = eeprom_byte_write_trace();
    session_send_logic_chunk(
        &mut session,
        0,
        &samples,
        DecodeExecutionLogicFormat::SplitLogic { unitsize: 1 },
        None,
    )
    .expect("stacked session should accept an EEPROM write trace");
    session
        .end()
        .expect("decode execution session should end cleanly");

    let annotations = session
        .take_captured_annotations()
        .expect("decode execution session should drain captured annotations");
    assert!(
        annotations.iter().any(|annotation| {
            annotation.decoder_id == "eeprom24xx"
                && annotation
                    .texts
                    .iter()
                    .any(|text| text.contains("Byte write"))
        }),
        "expected stacked decoder annotations after forwarding upstream OUTPUT_PYTHON data"
    );

    drop(session);
    runtime.exit().expect("decode runtime exit should succeed");
}

#[test]
fn synthetic_vcd_goldens_match_split_packet_fixture() {
    if skip_windows_vcd_goldens() {
        return;
    }
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };
    let sample_bytes = [0b00, 0b01, 0b10, 0b00];
    let packet_lengths = [2_usize, 2_usize];

    let export = runtime
        .render_vcd_from_logic_packets(&request, &sample_bytes, &packet_lengths, 1)
        .expect("split-packet replay should export VCD");
    let actual = normalize_vcd(&vcd_string(&export.bytes));
    let expected = fs::read_to_string(golden_path("two_channel_split_packets.vcd"))
        .expect("golden VCD should exist");

    assert_eq!(export.sample_count, 4);
    assert_eq!(export.packet_count, 4);
    assert_eq!(actual, expected);
}

#[test]
fn synthetic_vcd_goldens_lock_initial_values_deltas_and_final_timestamp() {
    if skip_windows_vcd_goldens() {
        return;
    }
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 250_000_000,
        enabled_channels: vec![3],
    };
    let sample_bytes = [0b0000, 0b1000, 0b0000, 0b1000];

    let export = runtime
        .render_vcd_from_samples(&request, &sample_bytes, 1)
        .expect("single-channel replay should export VCD");
    let actual = normalize_vcd(&vcd_string(&export.bytes));
    let expected = fs::read_to_string(golden_path("single_channel_final_timestamp.vcd"))
        .expect("golden VCD should exist");

    assert_eq!(export.sample_count, 4);
    assert_eq!(export.packet_count, 3);
    assert_eq!(actual, expected);
}

#[test]
fn synthetic_vcd_goldens_verify_transition_times_and_sample_count_semantics() {
    if skip_windows_vcd_goldens() {
        return;
    }
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };
    let sample_bytes = [0b00, 0b01, 0b10, 0b00];
    let packet_lengths = [2_usize, 2_usize];

    let export = runtime
        .render_vcd_from_logic_packets(&request, &sample_bytes, &packet_lengths, 1)
        .expect("split-packet replay should export VCD");
    let transitions = parse_transitions(&vcd_string(&export.bytes));

    assert_eq!(export.sample_count, sample_bytes.len() as u64);
    assert_eq!(
        transitions,
        vec![
            (0, "0!".to_string()),
            (0, "0\"".to_string()),
            (10, "1!".to_string()),
            (20, "0!".to_string()),
            (20, "1\"".to_string()),
            (30, "0\"".to_string()),
        ]
    );
    assert_eq!(final_timestamp(&vcd_string(&export.bytes)), Some(40));
}

#[test]
fn cross_logic_packet_replay_matches_expanded_sample_export() {
    if skip_windows_vcd_goldens() {
        return;
    }
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };

    let request = VcdExportRequest {
        samplerate_hz: 8_000_000,
        enabled_channels: vec![0, 1],
    };
    let ch0 = [0x0000_FFFF_0000_FFFF_u64, 0xF0F0_F0F0_0F0F_0F0F_u64];
    let ch1 = [0xAAAA_AAAA_5555_5555_u64, 0xCCCC_3333_CCCC_3333_u64];
    let cross_bytes = pack_cross_logic_blocks(&[&ch0, &ch1]);
    let expanded_samples = expand_cross_logic_blocks(&[&ch0, &ch1]);
    let packet_lengths = [16_usize, 16_usize];

    let cross_export = runtime
        .render_vcd_from_cross_logic_packets(&request, &cross_bytes, &packet_lengths)
        .expect("cross-logic replay should export VCD");
    let expanded_export = runtime
        .render_vcd_from_samples(&request, &expanded_samples, 1)
        .expect("expanded sample replay should export VCD");

    assert_eq!(cross_export.sample_count, 128);
    assert_eq!(expanded_export.sample_count, 128);
    assert_eq!(cross_export.packet_count, 4);
    assert_eq!(
        normalize_vcd(&vcd_string(&cross_export.bytes)),
        normalize_vcd(&vcd_string(&expanded_export.bytes))
    );
}

#[test]
fn synthetic_vcd_goldens_exercise_cleanup_safe_write_contract() {
    if skip_windows_vcd_goldens() {
        return;
    }
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 250_000_000,
        enabled_channels: vec![3],
    };
    let sample_bytes = [0b0000, 0b1000, 0b0000, 0b1000];
    let final_path = temp_path("cleanup-safe");
    let sibling_temp = final_path.with_extension("vcd.tmp");
    let _ = fs::remove_file(&final_path);
    let _ = fs::remove_file(&sibling_temp);

    let facts = runtime
        .render_vcd_from_samples_to_path(&request, &sample_bytes, 1, &final_path)
        .expect("render-to-path should succeed");

    assert!(final_path.exists());
    assert!(!sibling_temp.exists());
    assert_eq!(facts.sample_count, 4);
    assert_eq!(facts.packet_count, 3);
    assert_eq!(
        normalize_vcd(&fs::read_to_string(&final_path).expect("final VCD should exist")),
        fs::read_to_string(golden_path("single_channel_final_timestamp.vcd"))
            .expect("golden VCD should exist")
    );

    fs::remove_file(&final_path).expect("temp VCD cleanup should succeed");
}

#[test]
fn synthetic_vcd_goldens_reject_overflow_aligned_logic_packet_replay() {
    if skip_windows_vcd_goldens() {
        return;
    }
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };
    let sample_bytes = [0b00, 0b01, 0b10, 0b00];
    let invalid_packet_lengths = [3_usize, 1_usize];
    let final_path = temp_path("overflow-precondition");
    let sibling_temp = final_path.with_extension("vcd.tmp");
    let _ = fs::remove_file(&final_path);
    let _ = fs::remove_file(&sibling_temp);

    let error = runtime
        .render_vcd_from_logic_packets_to_path(
            &request,
            &sample_bytes,
            &invalid_packet_lengths,
            2,
            &final_path,
        )
        .expect_err("misaligned packet replay should fail fast");

    assert!(matches!(error, RuntimeError::InvalidArgument(detail) if detail.contains("aligned to unitsize")));
    assert!(!final_path.exists());
    assert!(!sibling_temp.exists());
}

#[test]
fn overflow_error_code_stays_stable_for_fail_fast_contract() {
    let error = RuntimeError::ExportCall {
        operation: "ds_export_recorded_vcd",
        code: ExportErrorCode::Overflow,
    };

    match error {
        RuntimeError::ExportCall { operation, code } => {
            assert_eq!(operation, "ds_export_recorded_vcd");
            assert_eq!(code, ExportErrorCode::Overflow);
            assert_eq!(code.name(), "export_overflow");
        }
        other => panic!("expected export overflow call error, got {other:?}"),
    }
}

#[test]
fn temp_promotion_failures_reference_temp_and_final_vcd_paths() {
    let error = RuntimeError::TempPromote {
        from: Path::new("/tmp/capture.vcd.tmp").to_path_buf(),
        to: Path::new("/tmp/capture.vcd").to_path_buf(),
        detail: "cross-device rename".to_string(),
    };

    match error {
        RuntimeError::TempPromote { from, to, detail } => {
            assert!(from.ends_with("capture.vcd.tmp"));
            assert!(to.ends_with("capture.vcd"));
            assert_eq!(detail, "cross-device rename");
        }
        other => panic!("expected temp promote failure, got {other:?}"),
    }
}
