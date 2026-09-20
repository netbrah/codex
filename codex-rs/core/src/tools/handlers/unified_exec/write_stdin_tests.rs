use super::*;
use pretty_assertions::assert_eq;

// Byte-identical copy of the private `expected an integer` message owned by
// codex-tools `strict_int` (see `strict_int_tests.rs` there). Drift pair:
// change both when one changes.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

#[test]
fn write_stdin_args_coerces_string_integers() {
    let args: WriteStdinArgs = serde_json::from_str(
        r#"{"session_id":"123","yield_time_ms":"250","max_output_tokens":"2000"}"#,
    )
    .expect("string integer literals should parse");
    assert_eq!(args.session_id, 123);
    assert_eq!(args.yield_time_ms, 250);
    assert_eq!(args.max_output_tokens, Some(2000));

    let args: WriteStdinArgs = serde_json::from_str(r#"{"session_id":"-5"}"#)
        .expect("negative strings should parse for signed fields");
    assert_eq!(args.session_id, -5);

    let args: WriteStdinArgs =
        serde_json::from_str(r#"{"session_id":"0123","yield_time_ms":"0250"}"#)
            .expect("leading zeros should parse");
    assert_eq!(args.session_id, 123);
    assert_eq!(args.yield_time_ms, 250);

    let args: WriteStdinArgs =
        serde_json::from_str(r#"{"session_id":-7,"yield_time_ms":300,"max_output_tokens":1024}"#)
            .expect("JSON numbers should still parse");
    assert_eq!(args.session_id, -7);
    assert_eq!(args.yield_time_ms, 300);
    assert_eq!(args.max_output_tokens, Some(1024));

    let args: WriteStdinArgs =
        serde_json::from_str(r#"{"session_id":42,"max_output_tokens":null}"#)
            .expect("null should map to None");
    assert_eq!(args.max_output_tokens, None);

    let err = serde_json::from_str::<WriteStdinArgs>(r#"{"session_id":1,"yield_time_ms":"-5"}"#)
        .expect_err("negative strings must be rejected for unsigned fields");
    assert!(
        err.to_string().contains(EXPECTED_INTEGER_ERROR),
        "negative unsigned input should carry the pinned integer error, got: {err}"
    );

    for bad in ["1.5", "12abc", " 12", "", "null", "+12", "2147483648"] {
        let err = serde_json::from_str::<WriteStdinArgs>(&format!(r#"{{"session_id":"{bad}"}}"#))
            .expect_err("malformed integer string should be rejected");
        assert!(
            err.to_string().contains(EXPECTED_INTEGER_ERROR),
            "input {bad:?} should carry the pinned integer error, got: {err}"
        );
    }

    for bad in ["1.5", "true", "[42]", r#"{"id":42}"#] {
        assert!(
            serde_json::from_str::<WriteStdinArgs>(&format!(r#"{{"session_id":{bad}}}"#)).is_err(),
            "non-integer JSON value {bad} should be rejected"
        );
    }
}

#[test]
fn write_stdin_args_rejects_unknown_field() {
    let err = serde_json::from_str::<WriteStdinArgs>(r#"{"session_id":1,"bogus":1}"#)
        .expect_err("unknown fields must be rejected");
    assert!(
        err.to_string().contains("unknown field `bogus`"),
        "unknown field error should name the field, got: {err}"
    );
}
