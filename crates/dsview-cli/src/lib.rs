pub mod capture_device_options;
pub mod device_options;

pub use capture_device_options::{
    CaptureTokenGuide, CaptureTokenLookupMaps, CliChannelModeOption, CliTokenOption,
    build_capture_token_guide, token_lookup_maps,
};
pub use device_options::{
    DeviceIdentityResponse, DeviceOptionsResponse, build_device_options_response,
    render_device_options_text,
};

#[cfg(test)]
mod tests {
    use super::{
        build_decode_inspect_response, build_decode_list_response, render_decode_inspect_text,
        render_decode_list_text,
    };
    use dsview_core::{
        DecoderAnnotationDescriptor, DecoderAnnotationRowDescriptor, DecoderChannelDescriptor,
        DecoderDescriptor, DecoderInputDescriptor, DecoderOptionDescriptor,
        DecoderOutputDescriptor,
    };
    use serde_json::json;

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
}
