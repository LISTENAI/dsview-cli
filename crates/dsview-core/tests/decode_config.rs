use dsview_core::{parse_decode_config, DecodeOptionValue};

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
