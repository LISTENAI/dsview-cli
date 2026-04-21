pub mod capture_device_options;
pub mod device_options;

use dsview_core::{
    DecodeFailureReport, DecodeReport, OfflineDecodeResult, OfflineDecodeRunError,
    DecoderAnnotationDescriptor, DecoderAnnotationRowDescriptor, DecoderChannelDescriptor,
    DecoderDescriptor, DecoderInputDescriptor, DecoderOptionDescriptor,
    DecoderOutputDescriptor,
};
pub use capture_device_options::{
    CaptureTokenGuide, CaptureTokenLookupMaps, CliChannelModeOption, CliTokenOption,
    build_capture_token_guide, token_lookup_maps,
};
pub use device_options::{
    DeviceIdentityResponse, DeviceOptionsResponse, build_device_options_response,
    render_device_options_text,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeListResponse {
    pub decoders: Vec<DecodeListEntryResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeListEntryResponse {
    pub id: String,
    pub name: String,
    pub longname: String,
    pub description: String,
    pub license: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub tags: Vec<String>,
    pub required_channel_ids: Vec<String>,
    pub optional_channel_ids: Vec<String>,
    pub option_ids: Vec<String>,
    pub annotation_ids: Vec<String>,
    pub annotation_row_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeInspectResponse {
    pub decoder: DecodeInspectDecoderResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeValidateResponse {
    pub valid: bool,
    pub config_version: u32,
    pub root_decoder_id: String,
    pub stack_depth: usize,
    pub bound_channel_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeInspectDecoderResponse {
    pub id: String,
    pub name: String,
    pub longname: String,
    pub description: String,
    pub license: String,
    pub inputs: Vec<DecodeIoResponse>,
    pub outputs: Vec<DecodeIoResponse>,
    pub tags: Vec<String>,
    pub required_channels: Vec<DecodeChannelResponse>,
    pub optional_channels: Vec<DecodeChannelResponse>,
    pub options: Vec<DecodeOptionResponse>,
    pub annotations: Vec<DecodeAnnotationResponse>,
    pub annotation_rows: Vec<DecodeAnnotationRowResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeIoResponse {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeChannelResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: i32,
    pub channel_type: i32,
    pub idn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeOptionResponse {
    pub id: String,
    pub idn: Option<String>,
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeAnnotationResponse {
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub annotation_type: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeAnnotationRowResponse {
    pub id: String,
    pub description: Option<String>,
    pub annotation_classes: Vec<usize>,
}

pub fn build_decode_list_response(decoders: &[DecoderDescriptor]) -> DecodeListResponse {
    DecodeListResponse {
        decoders: decoders
            .iter()
            .map(|decoder| DecodeListEntryResponse {
                id: decoder.id.clone(),
                name: decoder.name.clone(),
                longname: decoder.longname.clone(),
                description: decoder.description.clone(),
                license: decoder.license.clone(),
                inputs: decoder.inputs.iter().map(|input| input.id.clone()).collect(),
                outputs: decoder
                    .outputs
                    .iter()
                    .map(|output| output.id.clone())
                    .collect(),
                tags: decoder.tags.clone(),
                required_channel_ids: decoder
                    .required_channels
                    .iter()
                    .map(|channel| channel.id.clone())
                    .collect(),
                optional_channel_ids: decoder
                    .optional_channels
                    .iter()
                    .map(|channel| channel.id.clone())
                    .collect(),
                option_ids: decoder.options.iter().map(|option| option.id.clone()).collect(),
                annotation_ids: decoder
                    .annotations
                    .iter()
                    .map(|annotation| annotation.id.clone())
                    .collect(),
                annotation_row_ids: decoder
                    .annotation_rows
                    .iter()
                    .map(|row| row.id.clone())
                    .collect(),
            })
            .collect(),
    }
}

pub fn build_decode_inspect_response(decoder: &DecoderDescriptor) -> DecodeInspectResponse {
    DecodeInspectResponse {
        decoder: DecodeInspectDecoderResponse {
            id: decoder.id.clone(),
            name: decoder.name.clone(),
            longname: decoder.longname.clone(),
            description: decoder.description.clone(),
            license: decoder.license.clone(),
            inputs: decoder
                .inputs
                .iter()
                .map(decode_input_response)
                .collect(),
            outputs: decoder
                .outputs
                .iter()
                .map(decode_output_response)
                .collect(),
            tags: decoder.tags.clone(),
            required_channels: decoder
                .required_channels
                .iter()
                .map(decode_channel_response)
                .collect(),
            optional_channels: decoder
                .optional_channels
                .iter()
                .map(decode_channel_response)
                .collect(),
            options: decoder
                .options
                .iter()
                .map(decode_option_response)
                .collect(),
            annotations: decoder
                .annotations
                .iter()
                .map(decode_annotation_response)
                .collect(),
            annotation_rows: decoder
                .annotation_rows
                .iter()
                .map(decode_annotation_row_response)
                .collect(),
        },
    }
}

pub fn build_decode_validate_response(
    config_version: u32,
    root_decoder_id: impl Into<String>,
    bound_channel_ids: &[String],
    stack_depth: usize,
) -> DecodeValidateResponse {
    DecodeValidateResponse {
        valid: true,
        config_version,
        root_decoder_id: root_decoder_id.into(),
        stack_depth,
        bound_channel_ids: bound_channel_ids.to_vec(),
    }
}

pub fn build_decode_report_response(
    root_decoder_id: impl Into<String>,
    stack_depth: usize,
    sample_count: u64,
    result: &OfflineDecodeResult,
) -> DecodeReport {
    result.to_report(root_decoder_id, stack_depth, sample_count)
}

pub fn build_decode_failure_report_response(
    root_decoder_id: impl Into<String>,
    stack_depth: usize,
    sample_count: Option<u64>,
    error: &OfflineDecodeRunError,
) -> DecodeFailureReport {
    error.to_failure_report(root_decoder_id, stack_depth, sample_count)
}

pub fn render_decode_list_text(response: &DecodeListResponse) -> String {
    response
        .decoders
        .iter()
        .map(|decoder| {
            let mut lines = vec![format!(
                "{}\t{}\t{}",
                decoder.id, decoder.name, decoder.longname
            )];
            lines.push(format!(
                "  required: {}",
                join_ids(&decoder.required_channel_ids)
            ));
            lines.push(format!(
                "  optional: {}",
                join_ids(&decoder.optional_channel_ids)
            ));
            lines.push(format!("  inputs: {}", join_ids(&decoder.inputs)));
            lines.push(format!("  outputs: {}", join_ids(&decoder.outputs)));
            if !decoder.tags.is_empty() {
                lines.push(format!("  tags: {}", join_ids(&decoder.tags)));
            }
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn render_decode_inspect_text(response: &DecodeInspectResponse) -> String {
    let decoder = &response.decoder;
    let mut lines = vec![
        format!("decoder {}", decoder.id),
        format!("name: {}", decoder.name),
        format!("long name: {}", decoder.longname),
        format!("description: {}", decoder.description),
        format!("license: {}", decoder.license),
        format!(
            "inputs: {}",
            join_ids(&decoder.inputs.iter().map(|input| input.id.clone()).collect::<Vec<_>>())
        ),
        format!(
            "outputs: {}",
            join_ids(
                &decoder
                    .outputs
                    .iter()
                    .map(|output| output.id.clone())
                    .collect::<Vec<_>>()
            )
        ),
    ];
    if !decoder.tags.is_empty() {
        lines.push(format!("tags: {}", join_ids(&decoder.tags)));
    }

    lines.push("required channels:".to_string());
    lines.extend(render_channel_lines(&decoder.required_channels));

    lines.push("optional channels:".to_string());
    lines.extend(render_channel_lines(&decoder.optional_channels));

    lines.push("options:".to_string());
    if decoder.options.is_empty() {
        lines.push("  - none".to_string());
    } else {
        lines.extend(decoder.options.iter().map(|option| {
            let values = if option.values.is_empty() {
                "values: none".to_string()
            } else {
                format!("values: {}", join_ids(&option.values))
            };
            format!(
                "  - {} ({}) default={} {}",
                option.id,
                option
                    .description
                    .as_deref()
                    .unwrap_or("no description"),
                option.default_value.as_deref().unwrap_or("none"),
                values
            )
        }));
    }

    lines.push("annotations:".to_string());
    if decoder.annotations.is_empty() {
        lines.push("  - none".to_string());
    } else {
        lines.extend(decoder.annotations.iter().map(|annotation| {
            format!(
                "  - {} label={} description={}",
                annotation.id,
                annotation.label.as_deref().unwrap_or("none"),
                annotation.description.as_deref().unwrap_or("none")
            )
        }));
    }

    lines.push("annotation rows:".to_string());
    if decoder.annotation_rows.is_empty() {
        lines.push("  - none".to_string());
    } else {
        lines.extend(decoder.annotation_rows.iter().map(|row| {
            format!(
                "  - {} classes={} description={}",
                row.id,
                row.annotation_classes
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                row.description.as_deref().unwrap_or("none")
            )
        }));
    }

    lines.join("\n")
}

pub fn render_decode_validate_text(response: &DecodeValidateResponse) -> String {
    [
        "decode config valid".to_string(),
        format!("root decoder: {}", response.root_decoder_id),
        format!("config version: {}", response.config_version),
        format!("stack depth: {}", response.stack_depth),
        format!("bound channels: {}", join_ids(&response.bound_channel_ids)),
    ]
    .join("\n")
}

pub fn render_decode_report_text(response: &DecodeReport) -> String {
    [
        "decode run succeeded".to_string(),
        format!("root decoder: {}", response.run.root_decoder_id),
        format!("stack depth: {}", response.run.stack_depth),
        format!("sample count: {}", response.run.sample_count.unwrap_or(0)),
        format!("event count: {}", response.run.event_count.unwrap_or(0)),
    ]
    .join("\n")
}

pub fn render_decode_failure_report_text(response: &DecodeFailureReport) -> String {
    let mut lines = vec![
        "decode run failed".to_string(),
        format!("root decoder: {}", response.run.root_decoder_id),
        format!("stack depth: {}", response.run.stack_depth),
        format!(
            "partial event count: {}",
            response
                .diagnostics
                .as_ref()
                .map(|diagnostics| diagnostics.partial_event_count)
                .unwrap_or(response.partial_events.len())
        ),
    ];
    if let Some(sample_count) = response.run.sample_count {
        lines.push(format!("sample count: {sample_count}"));
    }
    lines.join("\n")
}

fn decode_input_response(input: &DecoderInputDescriptor) -> DecodeIoResponse {
    DecodeIoResponse {
        id: input.id.clone(),
    }
}

fn decode_output_response(output: &DecoderOutputDescriptor) -> DecodeIoResponse {
    DecodeIoResponse {
        id: output.id.clone(),
    }
}

fn decode_channel_response(channel: &DecoderChannelDescriptor) -> DecodeChannelResponse {
    DecodeChannelResponse {
        id: channel.id.clone(),
        name: channel.name.clone(),
        description: channel.description.clone(),
        order: channel.order,
        channel_type: channel.channel_type,
        idn: channel.idn.clone(),
    }
}

fn decode_option_response(option: &DecoderOptionDescriptor) -> DecodeOptionResponse {
    DecodeOptionResponse {
        id: option.id.clone(),
        idn: option.idn.clone(),
        description: option.description.clone(),
        default_value: option.default_value.clone(),
        values: option.values.clone(),
    }
}

fn decode_annotation_response(annotation: &DecoderAnnotationDescriptor) -> DecodeAnnotationResponse {
    DecodeAnnotationResponse {
        id: annotation.id.clone(),
        label: annotation.label.clone(),
        description: annotation.description.clone(),
        annotation_type: annotation.annotation_type,
    }
}

fn decode_annotation_row_response(
    row: &DecoderAnnotationRowDescriptor,
) -> DecodeAnnotationRowResponse {
    DecodeAnnotationRowResponse {
        id: row.id.clone(),
        description: row.description.clone(),
        annotation_classes: row.annotation_classes.clone(),
    }
}

fn render_channel_lines(channels: &[DecodeChannelResponse]) -> Vec<String> {
    if channels.is_empty() {
        return vec!["  - none".to_string()];
    }

    channels
        .iter()
        .map(|channel| {
            format!(
                "  - {} ({}) order={} type={} idn={}",
                channel.id,
                channel.name,
                channel.order,
                channel.channel_type,
                channel.idn.as_deref().unwrap_or("none")
            )
        })
        .collect()
}

fn join_ids(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_decode_inspect_response, build_decode_list_response,
        build_decode_failure_report_response, build_decode_report_response,
        build_decode_validate_response, render_decode_failure_report_text,
        render_decode_inspect_text, render_decode_list_text, render_decode_report_text,
        render_decode_validate_text,
    };
    use dsview_core::{
        run_offline_decode, DecodeCapturedAnnotation, DecodeRunStatus, DecodeRuntimeError,
        DecoderAnnotationDescriptor, DecoderAnnotationRowDescriptor, DecoderChannelDescriptor,
        DecoderDescriptor, DecoderInputDescriptor, DecoderOptionDescriptor,
        DecoderOutputDescriptor, OfflineDecodeDataFormat, OfflineDecodeInput,
        OfflineDecodeRuntime, OfflineDecodeRuntimeSession, ValidatedDecodeConfig,
        ValidatedDecodeDecoderConfig, ValidatedDecodeStackEntryConfig,
    };
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::{BTreeMap, VecDeque};
    use std::rc::Rc;

    #[derive(Debug, Default)]
    struct RecordingState {
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
        fn with_send_and_end(
            send_responses: Vec<SessionResponse>,
            end_response: SessionResponse,
        ) -> Self {
            Self {
                state: Rc::new(RefCell::new(RecordingState {
                    send_responses: send_responses.into(),
                    end_response: Some(end_response),
                })),
            }
        }

        fn with_send_responses(send_responses: Vec<SessionResponse>) -> Self {
            Self {
                state: Rc::new(RefCell::new(RecordingState {
                    send_responses: send_responses.into(),
                    end_response: None,
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
            _root: &dsview_core::DecodeSessionInstance,
            _stack: &[dsview_core::DecodeSessionInstance],
        ) -> Result<(), DecodeRuntimeError> {
            Ok(())
        }

        fn start(&mut self) -> Result<(), DecodeRuntimeError> {
            Ok(())
        }

        fn send_logic_chunk(
            &mut self,
            _abs_start_sample: u64,
            _sample_bytes: &[u8],
            _format: dsview_core::DecodeExecutionLogicFormat,
        ) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError> {
            match self.state.borrow_mut().send_responses.pop_front() {
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

    fn sample_decoder() -> DecoderDescriptor {
        DecoderDescriptor {
            id: "0:i2c".to_string(),
            name: "i2c".to_string(),
            longname: "Inter-Integrated Circuit".to_string(),
            description: "Two-wire serial bus".to_string(),
            license: "gplv2+".to_string(),
            inputs: vec![DecoderInputDescriptor {
                id: "logic".to_string(),
            }],
            outputs: vec![
                DecoderOutputDescriptor {
                    id: "i2c".to_string(),
                },
                DecoderOutputDescriptor {
                    id: "i2c-messages".to_string(),
                },
            ],
            tags: vec!["serial".to_string(), "embedded".to_string()],
            required_channels: vec![DecoderChannelDescriptor {
                id: "scl".to_string(),
                name: "SCL".to_string(),
                description: "Clock".to_string(),
                order: 0,
                channel_type: 0,
                idn: Some("clk".to_string()),
            }],
            optional_channels: vec![DecoderChannelDescriptor {
                id: "sda".to_string(),
                name: "SDA".to_string(),
                description: "Data".to_string(),
                order: 1,
                channel_type: 0,
                idn: Some("data".to_string()),
            }],
            options: vec![DecoderOptionDescriptor {
                id: "address_format".to_string(),
                idn: Some("address_format".to_string()),
                description: Some("Whether addresses render as 7-bit or 8-bit".to_string()),
                value_kind: dsview_core::DecodeOptionValueKind::String,
                default_value: Some("7-bit".to_string()),
                values: vec!["7-bit".to_string(), "8-bit".to_string()],
            }],
            annotations: vec![DecoderAnnotationDescriptor {
                id: "start".to_string(),
                label: Some("START".to_string()),
                description: Some("Start condition".to_string()),
                annotation_type: 0,
            }],
            annotation_rows: vec![DecoderAnnotationRowDescriptor {
                id: "frames".to_string(),
                description: Some("Frame events".to_string()),
                annotation_classes: vec![0],
            }],
        }
    }

    #[test]
    fn decode_list_response_and_text_preserve_canonical_ids() {
        let response = build_decode_list_response(&[sample_decoder()]);
        let value = serde_json::to_value(&response).expect("list response should serialize");

        assert_eq!(
            value,
            json!({
                "decoders": [{
                    "id": "0:i2c",
                    "name": "i2c",
                    "longname": "Inter-Integrated Circuit",
                    "description": "Two-wire serial bus",
                    "license": "gplv2+",
                    "inputs": ["logic"],
                    "outputs": ["i2c", "i2c-messages"],
                    "tags": ["serial", "embedded"],
                    "required_channel_ids": ["scl"],
                    "optional_channel_ids": ["sda"],
                    "option_ids": ["address_format"],
                    "annotation_ids": ["start"],
                    "annotation_row_ids": ["frames"]
                }]
            })
        );

        let text = render_decode_list_text(&response);
        assert!(text.contains("0:i2c"));
        assert!(text.contains("Inter-Integrated Circuit"));
        assert!(text.contains("required: scl"));
        assert!(text.contains("outputs: i2c, i2c-messages"));
    }

    #[test]
    fn decode_inspect_response_and_text_include_stack_metadata() {
        let response = build_decode_inspect_response(&sample_decoder());
        let value = serde_json::to_value(&response).expect("inspect response should serialize");

        assert_eq!(value["decoder"]["id"], "0:i2c");
        assert_eq!(value["decoder"]["required_channels"][0]["id"], "scl");
        assert_eq!(value["decoder"]["optional_channels"][0]["id"], "sda");
        assert_eq!(value["decoder"]["options"][0]["id"], "address_format");
        assert_eq!(value["decoder"]["annotations"][0]["id"], "start");
        assert_eq!(value["decoder"]["annotation_rows"][0]["id"], "frames");
        assert_eq!(value["decoder"]["inputs"][0]["id"], "logic");
        assert_eq!(value["decoder"]["outputs"][1]["id"], "i2c-messages");

        let text = render_decode_inspect_text(&response);
        assert!(text.contains("decoder 0:i2c"));
        assert!(text.contains("required channels"));
        assert!(text.contains("optional channels"));
        assert!(text.contains("options"));
        assert!(text.contains("annotation rows"));
        assert!(text.contains("inputs: logic"));
        assert!(text.contains("outputs: i2c, i2c-messages"));
    }

    #[test]
    fn decode_validate_response_and_text_summarize_valid_config() {
        let response = build_decode_validate_response(
            1,
            "0:i2c",
            &["scl".to_string(), "sda".to_string()],
            1,
        );
        let value = serde_json::to_value(&response).expect("validate response should serialize");

        assert_eq!(
            value,
            json!({
                "valid": true,
                "config_version": 1,
                "root_decoder_id": "0:i2c",
                "stack_depth": 1,
                "bound_channel_ids": ["scl", "sda"]
            })
        );

        let text = render_decode_validate_text(&response);
        assert!(text.contains("decode config valid"));
        assert!(text.contains("root decoder: 0:i2c"));
        assert!(text.contains("stack depth: 1"));
        assert!(text.contains("bound channels: scl, sda"));
    }

    #[test]
    fn decode_report_response_and_text_summarize_execution() {
        let runtime = RecordingRuntime::with_send_and_end(
            vec![SessionResponse::Ok(vec![DecodeCapturedAnnotation {
                decoder_id: "0:i2c".to_string(),
                start_sample: 0,
                end_sample: 2,
                annotation_class: 0,
                annotation_type: 10,
                texts: vec!["start".to_string()],
            }])],
            SessionResponse::Ok(vec![DecodeCapturedAnnotation {
                decoder_id: "eeprom24xx".to_string(),
                start_sample: 2,
                end_sample: 4,
                annotation_class: 1,
                annotation_type: 20,
                texts: vec!["write".to_string()],
            }]),
        );
        let input = fixture_input();
        let result = run_offline_decode(&fixture_config(), &input, &runtime)
            .expect("run response should build from successful execution");
        let response = build_decode_report_response("0:i2c", 1, input.sample_count().unwrap(), &result);
        let value = serde_json::to_value(&response).expect("run response should serialize");

        assert_eq!(
            value,
            json!({
                "run": {
                    "status": "success",
                    "root_decoder_id": "0:i2c",
                    "stack_depth": 1,
                    "sample_count": 4,
                    "event_count": 2
                },
                "events": [
                    {
                        "decoder_id": "0:i2c",
                        "start_sample": 0,
                        "end_sample": 2,
                        "annotation_class": 0,
                        "annotation_type": 10,
                        "texts": ["start"]
                    },
                    {
                        "decoder_id": "eeprom24xx",
                        "start_sample": 2,
                        "end_sample": 4,
                        "annotation_class": 1,
                        "annotation_type": 20,
                        "texts": ["write"]
                    }
                ]
            })
        );

        let text = render_decode_report_text(&response);
        assert!(text.contains("decode run succeeded"));
        assert!(text.contains("root decoder: 0:i2c"));
        assert!(text.contains("stack depth: 1"));
        assert!(text.contains("sample count: 4"));
        assert!(text.contains("event count: 2"));
    }

    #[test]
    fn decode_failure_report_response_and_text_preserve_partial_events() {
        let retained = DecodeCapturedAnnotation {
            decoder_id: "eeprom24xx".to_string(),
            start_sample: 0,
            end_sample: 2,
            annotation_class: 1,
            annotation_type: 20,
            texts: vec!["write".to_string()],
        };
        let runtime = RecordingRuntime::with_send_responses(vec![
            SessionResponse::Ok(vec![retained.clone()]),
            SessionResponse::Err("second chunk failed"),
        ]);
        let input = fixture_input();
        let error = run_offline_decode(&fixture_config(), &input, &runtime)
            .expect_err("fixture should fail after partial output");
        let response = build_decode_failure_report_response(
            "0:i2c",
            1,
            Some(input.sample_count().unwrap()),
            &error,
        );
        let value = serde_json::to_value(&response).expect("failure response should serialize");

        assert_eq!(response.run.status, DecodeRunStatus::Failure);
        assert_eq!(
            value,
            json!({
                "run": {
                    "status": "failure",
                    "root_decoder_id": "0:i2c",
                    "stack_depth": 1,
                    "sample_count": 4
                },
                "partial_events": [
                    {
                        "decoder_id": "eeprom24xx",
                        "start_sample": 0,
                        "end_sample": 2,
                        "annotation_class": 1,
                        "annotation_type": 20,
                        "texts": ["write"]
                    }
                ],
                "diagnostics": {
                    "completed_chunks": 1,
                    "consumed_samples": 2,
                    "partial_event_count": 1,
                    "partial_events_available": true
                }
            })
        );

        let text = render_decode_failure_report_text(&response);
        assert!(text.contains("decode run failed"));
        assert!(text.contains("root decoder: 0:i2c"));
        assert!(text.contains("partial event count: 1"));
        assert!(text.contains("sample count: 4"));
    }

    fn fixture_input() -> OfflineDecodeInput {
        OfflineDecodeInput {
            samplerate_hz: 1_000_000,
            format: OfflineDecodeDataFormat::SplitLogic,
            sample_bytes: vec![0x10, 0x11, 0x12, 0x13],
            unitsize: 1,
            channel_count: None,
            logic_packet_lengths: Some(vec![2, 2]),
        }
    }

    fn fixture_config() -> ValidatedDecodeConfig {
        ValidatedDecodeConfig {
            version: 1,
            decoder: ValidatedDecodeDecoderConfig {
                descriptor: sample_decoder(),
                channels: BTreeMap::from([
                    ("scl".to_string(), 0_u32),
                    ("sda".to_string(), 1_u32),
                ]),
                options: BTreeMap::new(),
            },
            stack: vec![ValidatedDecodeStackEntryConfig {
                descriptor: DecoderDescriptor {
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
                },
                options: BTreeMap::new(),
            }],
        }
    }
}
