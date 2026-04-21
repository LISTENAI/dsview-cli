use dsview_core::{
    parse_decode_config, validate_decode_config, DecodeConfigValidationError, DecodeOptionValue,
    DecodeOptionValueKind, DecoderChannelDescriptor, DecoderDescriptor, DecoderInputDescriptor,
    DecoderOptionDescriptor, DecoderOutputDescriptor, ValidatedDecodeConfig,
};

#[test]
fn parse_decode_config_with_linear_stack() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "0:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            },
            "stack": [
                {
                    "id": "eeprom24xx",
                    "options": {
                        "addr_counter": 0
                    }
                }
            ]
        }"#,
    )
    .expect("config should parse");

    assert_eq!(config.version, 1);
    assert_eq!(config.decoder.id, "0:i2c");
    assert_eq!(config.stack.len(), 1);
    assert_eq!(config.stack[0].id, "eeprom24xx");
}

#[test]
fn channel_bindings_use_numeric_indexes() {
    let config = parse_decode_config(
        r#"{
            "version": 1,
            "decoder": {
                "id": "0:spi",
                "channels": {
                    "clk": 0,
                    "miso": 3,
                    "mosi": 2
                },
                "options": {}
            },
            "stack": []
        }"#,
    )
    .expect("config should parse");

    assert_eq!(config.decoder.channels["clk"], 0);
    assert_eq!(config.decoder.channels["mosi"], 2);
    assert_eq!(config.decoder.channels["miso"], 3);
}

#[test]
fn option_values_preserve_typed_json_shape() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "can",
                "channels": {
                    "can_rx": 0
                },
                "options": {
                    "address_format": "shifted",
                    "bitrate": 1000000,
                    "sample_point": 70.0
                }
            },
            "stack": [
                {
                    "id": "numbers_and_state",
                    "options": {
                        "format": "hex",
                        "count": 8,
                        "scale": 1.5
                    }
                }
            ]
        }"#,
    )
    .expect("config should parse");

    assert_eq!(
        config.decoder.options["address_format"],
        DecodeOptionValue::String("shifted".to_string())
    );
    assert_eq!(
        config.decoder.options["bitrate"],
        DecodeOptionValue::Integer(1_000_000)
    );
    assert_eq!(
        config.decoder.options["sample_point"],
        DecodeOptionValue::Float(70.0)
    );
    assert_eq!(
        config.stack[0].options["count"],
        DecodeOptionValue::Integer(8)
    );
    assert_eq!(
        config.stack[0].options["scale"],
        DecodeOptionValue::Float(1.5)
    );
}

#[test]
fn rejects_missing_required_channels() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "0:i2c",
                "channels": {
                    "scl": 0
                },
                "options": {
                    "address_format": "unshifted"
                }
            }
        }"#,
    )
    .expect("config should parse");

    let error = validate_decode_config(&config, &decoder_registry()).expect_err("validation should fail");

    assert_eq!(
        error,
        DecodeConfigValidationError::MissingRequiredChannel {
            decoder_id: "0:i2c".to_string(),
            channel_id: "sda".to_string(),
        }
    );
}

#[test]
fn rejects_unknown_option_ids() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "0:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "unknown_option": "oops"
                }
            }
        }"#,
    )
    .expect("config should parse");

    let error = validate_decode_config(&config, &decoder_registry()).expect_err("validation should fail");

    assert_eq!(
        error,
        DecodeConfigValidationError::UnknownOption {
            decoder_id: "0:i2c".to_string(),
            option_id: "unknown_option".to_string(),
        }
    );
}

#[test]
fn rejects_option_value_type_mismatch() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "0:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": 1
                }
            }
        }"#,
    )
    .expect("config should parse");

    let error = validate_decode_config(&config, &decoder_registry()).expect_err("validation should fail");

    assert_eq!(
        error,
        DecodeConfigValidationError::InvalidOptionValueType {
            decoder_id: "0:i2c".to_string(),
            option_id: "address_format".to_string(),
            expected: DecodeOptionValueKind::String,
            actual: DecodeOptionValueKind::Integer,
        }
    );
}

#[test]
fn rejects_incompatible_linear_stack() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "0:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            },
            "stack": [
                {
                    "id": "spi-frames",
                    "options": {}
                }
            ]
        }"#,
    )
    .expect("config should parse");

    let error = validate_decode_config(&config, &decoder_registry()).expect_err("validation should fail");

    assert_eq!(
        error,
        DecodeConfigValidationError::IncompatibleStackLink {
            upstream_decoder_id: "0:i2c".to_string(),
            downstream_decoder_id: "spi-frames".to_string(),
            upstream_outputs: vec!["i2c".to_string(), "i2c-messages".to_string()],
            downstream_inputs: vec!["spi".to_string()],
        }
    );
}

#[test]
fn accepts_valid_linear_stack_against_decoder_metadata() {
    let config = parse_decode_config(
        r#"{
            "decoder": {
                "id": "0:i2c",
                "channels": {
                    "scl": 0,
                    "sda": 1
                },
                "options": {
                    "address_format": "unshifted"
                }
            },
            "stack": [
                {
                    "id": "eeprom24xx",
                    "options": {
                        "addr_counter": 0
                    }
                }
            ]
        }"#,
    )
    .expect("config should parse");

    let validated =
        validate_decode_config(&config, &decoder_registry()).expect("validation should pass");

    assert_validated_linear_stack(&validated);
}

fn assert_validated_linear_stack(validated: &ValidatedDecodeConfig) {
    assert_eq!(validated.decoder.descriptor.id, "0:i2c");
    assert_eq!(validated.decoder.channels["scl"], 0);
    assert_eq!(
        validated.decoder.options["address_format"],
        DecodeOptionValue::String("unshifted".to_string())
    );
    assert_eq!(validated.stack.len(), 1);
    assert_eq!(validated.stack[0].descriptor.id, "eeprom24xx");
    assert_eq!(
        validated.stack[0].options["addr_counter"],
        DecodeOptionValue::Integer(0)
    );
}

fn decoder_registry() -> Vec<DecoderDescriptor> {
    vec![
        DecoderDescriptor {
            id: "0:i2c".to_string(),
            name: "i2c".to_string(),
            longname: "I2C".to_string(),
            description: "I2C decoder".to_string(),
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
            tags: vec!["serial".to_string()],
            required_channels: vec![
                decoder_channel("scl", 0),
                decoder_channel("sda", 1),
            ],
            optional_channels: vec![],
            options: vec![DecoderOptionDescriptor {
                id: "address_format".to_string(),
                idn: Some("address_format".to_string()),
                description: Some("Address display format".to_string()),
                value_kind: DecodeOptionValueKind::String,
                default_value: Some("shifted".to_string()),
                values: vec!["shifted".to_string(), "unshifted".to_string()],
            }],
            annotations: vec![],
            annotation_rows: vec![],
        },
        DecoderDescriptor {
            id: "eeprom24xx".to_string(),
            name: "eeprom24xx".to_string(),
            longname: "EEPROM 24xx".to_string(),
            description: "EEPROM decoder".to_string(),
            license: "gplv2+".to_string(),
            inputs: vec![DecoderInputDescriptor {
                id: "i2c".to_string(),
            }],
            outputs: vec![DecoderOutputDescriptor {
                id: "eeprom24xx".to_string(),
            }],
            tags: vec!["memory".to_string()],
            required_channels: vec![],
            optional_channels: vec![],
            options: vec![DecoderOptionDescriptor {
                id: "addr_counter".to_string(),
                idn: Some("addr_counter".to_string()),
                description: Some("Address counter".to_string()),
                value_kind: DecodeOptionValueKind::Integer,
                default_value: Some("0".to_string()),
                values: vec![],
            }],
            annotations: vec![],
            annotation_rows: vec![],
        },
        DecoderDescriptor {
            id: "spi-frames".to_string(),
            name: "spi-frames".to_string(),
            longname: "SPI frames".to_string(),
            description: "SPI frame decoder".to_string(),
            license: "gplv2+".to_string(),
            inputs: vec![DecoderInputDescriptor {
                id: "spi".to_string(),
            }],
            outputs: vec![],
            tags: vec!["serial".to_string()],
            required_channels: vec![],
            optional_channels: vec![],
            options: vec![],
            annotations: vec![],
            annotation_rows: vec![],
        },
    ]
}

fn decoder_channel(id: &str, order: i32) -> DecoderChannelDescriptor {
    DecoderChannelDescriptor {
        id: id.to_string(),
        name: id.to_string(),
        description: format!("{id} line"),
        order,
        channel_type: 0,
        idn: Some(id.to_string()),
    }
}
