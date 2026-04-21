use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::rc::Rc;

use dsview_core::{
    run_offline_decode, DecodeCapturedAnnotation, DecodeExecutionLogicFormat, DecodeOptionValue,
    DecodeRuntimeError, DecoderChannelDescriptor, DecoderDescriptor, DecoderInputDescriptor,
    DecoderOutputDescriptor, OfflineDecodeDataFormat, OfflineDecodeInput, OfflineDecodeRuntime,
    OfflineDecodeRuntimeSession, ValidatedDecodeConfig, ValidatedDecodeDecoderConfig,
    ValidatedDecodeStackEntryConfig, OFFLINE_DECODE_FIXED_CHUNK_BYTES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedChunk {
    abs_start_sample: u64,
    sample_len: usize,
    format: DecodeExecutionLogicFormat,
}

#[derive(Debug, Default)]
struct RecordingState {
    root: Option<dsview_core::DecodeSessionInstance>,
    stack: Vec<dsview_core::DecodeSessionInstance>,
    chunks: Vec<RecordedChunk>,
    send_responses: VecDeque<SessionResponse>,
    end_response: Option<SessionResponse>,
}

#[derive(Clone, Default)]
struct RecordingRuntime {
    state: Rc<RefCell<RecordingState>>,
}

struct RecordingSession {
    state: Rc<RefCell<RecordingState>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SessionResponse {
    Ok(Vec<DecodeCapturedAnnotation>),
    Err(&'static str),
}

impl RecordingRuntime {
    fn with_send_responses(send_responses: Vec<SessionResponse>) -> Self {
        Self {
            state: Rc::new(RefCell::new(RecordingState {
                send_responses: send_responses.into(),
                ..RecordingState::default()
            })),
        }
    }

    fn with_end_response(end_response: SessionResponse) -> Self {
        Self {
            state: Rc::new(RefCell::new(RecordingState {
                end_response: Some(end_response),
                ..RecordingState::default()
            })),
        }
    }

    fn with_send_and_end(
        send_responses: Vec<SessionResponse>,
        end_response: SessionResponse,
    ) -> Self {
        Self {
            state: Rc::new(RefCell::new(RecordingState {
                send_responses: send_responses.into(),
                end_response: Some(end_response),
                ..RecordingState::default()
            })),
        }
    }
}

impl OfflineDecodeRuntime for RecordingRuntime {
    type Session = RecordingSession;

    fn create_session(&self) -> Result<Self::Session, DecodeRuntimeError> {
        Ok(RecordingSession {
            state: Rc::clone(&self.state),
        })
    }
}

impl OfflineDecodeRuntimeSession for RecordingSession {
    fn set_samplerate_hz(&mut self, _samplerate_hz: u64) -> Result<(), DecodeRuntimeError> {
        Ok(())
    }

    fn build_linear_stack(
        &mut self,
        root: &dsview_core::DecodeSessionInstance,
        stack: &[dsview_core::DecodeSessionInstance],
    ) -> Result<(), DecodeRuntimeError> {
        let mut state = self.state.borrow_mut();
        state.root = Some(root.clone());
        state.stack = stack.to_vec();
        Ok(())
    }

    fn start(&mut self) -> Result<(), DecodeRuntimeError> {
        Ok(())
    }

    fn send_logic_chunk(
        &mut self,
        abs_start_sample: u64,
        sample_bytes: &[u8],
        format: DecodeExecutionLogicFormat,
    ) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError> {
        let mut state = self.state.borrow_mut();
        state.chunks.push(RecordedChunk {
            abs_start_sample,
            sample_len: sample_bytes.len(),
            format,
        });
        match state.send_responses.pop_front() {
            Some(SessionResponse::Ok(annotations)) => Ok(annotations),
            Some(SessionResponse::Err(detail)) => {
                Err(DecodeRuntimeError::InvalidArgument(detail.to_string()))
            }
            None => Ok(Vec::new()),
        }
    }

    fn end(&mut self) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError> {
        match self.state.borrow_mut().end_response.take() {
            Some(SessionResponse::Ok(annotations)) => Ok(annotations),
            Some(SessionResponse::Err(detail)) => {
                Err(DecodeRuntimeError::InvalidArgument(detail.to_string()))
            }
            None => Ok(Vec::new()),
        }
    }
}

#[test]
fn offline_decode_uses_absolute_sample_cursor_across_chunks() {
    let runtime = RecordingRuntime::default();
    let input = OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0x55; (OFFLINE_DECODE_FIXED_CHUNK_BYTES * 2) + 3],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: None,
    };

    let result =
        run_offline_decode(&validated_decode_config(), &input, &runtime).expect("decode run should succeed");

    assert!(result.annotations().is_empty());

    let state = runtime.state.borrow();
    assert_eq!(
        state
            .chunks
            .iter()
            .map(|chunk| chunk.abs_start_sample)
            .collect::<Vec<_>>(),
        vec![
            0,
            OFFLINE_DECODE_FIXED_CHUNK_BYTES as u64,
            (OFFLINE_DECODE_FIXED_CHUNK_BYTES * 2) as u64,
        ]
    );
}

#[test]
fn offline_decode_prefers_packet_boundaries_when_available() {
    let runtime = RecordingRuntime::default();
    let input = OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0x10, 0x11, 0x12, 0x13, 0x14, 0x15],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: Some(vec![1, 2, 3]),
    };

    run_offline_decode(&validated_decode_config(), &input, &runtime)
        .expect("decode run should succeed");

    let state = runtime.state.borrow();
    assert_eq!(
        state
            .chunks
            .iter()
            .map(|chunk| (chunk.abs_start_sample, chunk.sample_len))
            .collect::<Vec<_>>(),
        vec![(0, 1), (1, 2), (3, 3)]
    );
}

#[test]
fn offline_decode_root_only_binds_logic_channels() {
    let runtime = RecordingRuntime::default();
    let input = OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0x00, 0x01, 0x02, 0x03],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: None,
    };

    run_offline_decode(&validated_decode_config(), &input, &runtime)
        .expect("decode run should succeed");

    let state = runtime.state.borrow();
    let root = state.root.as_ref().expect("root instance should be recorded");
    assert_eq!(root.channel_bindings.len(), 2);
    assert_eq!(state.stack.len(), 1);
    assert!(state.stack[0].channel_bindings.is_empty());
}

#[test]
fn offline_decode_fails_when_session_send_fails() {
    let runtime = RecordingRuntime::with_send_responses(vec![SessionResponse::Err("send exploded")]);
    let input = OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0xAA, 0xBB],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: None,
    };

    let error = run_offline_decode(&validated_decode_config(), &input, &runtime)
        .expect_err("send failures should fail the decode run");

    assert_eq!(error.operation(), "send logic chunk");
    assert_eq!(error.completed_chunks(), 0);
}

#[test]
fn offline_decode_fails_when_session_end_fails() {
    let runtime = RecordingRuntime::with_end_response(SessionResponse::Err("end exploded"));
    let input = OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0xAA, 0xBB],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: None,
    };

    let error = run_offline_decode(&validated_decode_config(), &input, &runtime)
        .expect_err("end failures should fail the decode run");

    assert_eq!(error.operation(), "end session");
    assert_eq!(error.completed_chunks(), 1);
}

#[test]
fn offline_decode_retains_partial_annotations_for_diagnostics_only() {
    let retained = DecodeCapturedAnnotation {
        decoder_id: "eeprom24xx".to_string(),
        start_sample: 0,
        end_sample: 2,
        annotation_class: 9,
        annotation_type: 9,
        texts: vec!["Byte write".to_string()],
    };
    let runtime = RecordingRuntime::with_send_responses(vec![
        SessionResponse::Ok(vec![retained.clone()]),
        SessionResponse::Err("second chunk failed"),
    ]);
    let input = OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0x01, 0x02, 0x03, 0x04],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: Some(vec![2, 2]),
    };

    let error = run_offline_decode(&validated_decode_config(), &input, &runtime)
        .expect_err("send failures should keep the run in a binary failure state");

    assert_eq!(error.operation(), "send logic chunk");
    assert_eq!(error.completed_chunks(), 1);
    assert_eq!(error.retained_annotations(), &[retained]);
}

#[test]
fn offline_decode_collects_fixture_annotations_for_successful_runs() {
    let root_annotation = DecodeCapturedAnnotation {
        decoder_id: "fixture:i2c".to_string(),
        start_sample: 0,
        end_sample: 2,
        annotation_class: 0,
        annotation_type: 0,
        texts: vec!["chunk-1".to_string()],
    };
    let stack_annotation = DecodeCapturedAnnotation {
        decoder_id: "fixture:eeprom24xx".to_string(),
        start_sample: 0,
        end_sample: 2,
        annotation_class: 1,
        annotation_type: 1,
        texts: vec!["fixture-complete".to_string()],
    };
    let runtime = RecordingRuntime::with_send_and_end(
        vec![
            SessionResponse::Ok(vec![root_annotation.clone()]),
            SessionResponse::Ok(vec![root_annotation.clone()]),
        ],
        SessionResponse::Ok(vec![stack_annotation.clone()]),
    );

    let result = run_offline_decode(&validated_decode_config(), &fixture_cli_input(), &runtime)
        .expect("decode run should succeed");

    assert_eq!(
        result.annotations(),
        &[
            root_annotation.clone(),
            root_annotation,
            stack_annotation,
        ]
    );
}

fn fixture_cli_input() -> OfflineDecodeInput {
    OfflineDecodeInput {
        samplerate_hz: 1_000_000,
        format: OfflineDecodeDataFormat::SplitLogic,
        sample_bytes: vec![0x10, 0x11, 0x12, 0x13],
        unitsize: 1,
        channel_count: None,
        logic_packet_lengths: Some(vec![2, 2]),
    }
}

fn validated_decode_config() -> ValidatedDecodeConfig {
    ValidatedDecodeConfig {
        version: 1,
        decoder: ValidatedDecodeDecoderConfig {
            descriptor: root_decoder_descriptor(),
            channels: BTreeMap::from([
                ("scl".to_string(), 0_u32),
                ("sda".to_string(), 1_u32),
            ]),
            options: BTreeMap::from([(
                "address_format".to_string(),
                DecodeOptionValue::String("unshifted".to_string()),
            )]),
        },
        stack: vec![ValidatedDecodeStackEntryConfig {
            descriptor: stacked_decoder_descriptor(),
            options: BTreeMap::new(),
        }],
    }
}

fn root_decoder_descriptor() -> DecoderDescriptor {
    DecoderDescriptor {
        id: "0:i2c".to_string(),
        name: "i2c".to_string(),
        longname: "Inter-Integrated Circuit".to_string(),
        description: "fixture root decoder".to_string(),
        license: "gplv2+".to_string(),
        inputs: vec![DecoderInputDescriptor {
            id: "logic".to_string(),
        }],
        outputs: vec![DecoderOutputDescriptor {
            id: "i2c".to_string(),
        }],
        tags: vec!["serial".to_string()],
        required_channels: vec![
            DecoderChannelDescriptor {
                id: "scl".to_string(),
                name: "SCL".to_string(),
                description: "Serial clock".to_string(),
                order: 0,
                channel_type: 8,
                idn: None,
            },
            DecoderChannelDescriptor {
                id: "sda".to_string(),
                name: "SDA".to_string(),
                description: "Serial data".to_string(),
                order: 1,
                channel_type: 108,
                idn: None,
            },
        ],
        optional_channels: Vec::new(),
        options: Vec::new(),
        annotations: Vec::new(),
        annotation_rows: Vec::new(),
    }
}

fn stacked_decoder_descriptor() -> DecoderDescriptor {
    DecoderDescriptor {
        id: "eeprom24xx".to_string(),
        name: "24xx EEPROM".to_string(),
        longname: "24xx I2C EEPROM".to_string(),
        description: "fixture stack decoder".to_string(),
        license: "gplv2+".to_string(),
        inputs: vec![DecoderInputDescriptor {
            id: "i2c".to_string(),
        }],
        outputs: Vec::new(),
        tags: vec!["memory".to_string()],
        required_channels: Vec::new(),
        optional_channels: Vec::new(),
        options: Vec::new(),
        annotations: Vec::new(),
        annotation_rows: Vec::new(),
    }
}
