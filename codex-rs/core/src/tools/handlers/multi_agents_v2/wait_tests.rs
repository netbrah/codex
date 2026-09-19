use super::*;
use pretty_assertions::assert_eq;

// Byte-identical copy of the private `expected an integer` message owned by
// codex-tools `strict_int` (see `strict_int_tests.rs` there). Drift pair:
// change both when one changes.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

#[test]
fn wait_agent_v2_args_coerces_string_integers() {
    let args: WaitArgs = serde_json::from_str(r#"{"timeout_ms":"5000"}"#)
        .expect("string integer literals should parse");
    assert_eq!(args.timeout_ms, Some(5000));

    let args: WaitArgs = serde_json::from_str(r#"{"timeout_ms":"-5"}"#)
        .expect("negative strings should parse for signed fields");
    assert_eq!(args.timeout_ms, Some(-5));

    let args: WaitArgs =
        serde_json::from_str(r#"{"timeout_ms":"0123"}"#).expect("leading zeros should parse");
    assert_eq!(args.timeout_ms, Some(123));

    let args: WaitArgs =
        serde_json::from_str(r#"{"timeout_ms":7500}"#).expect("JSON numbers should still parse");
    assert_eq!(args.timeout_ms, Some(7500));

    let args: WaitArgs = serde_json::from_str(r#"{}"#).expect("missing Option field should parse");
    assert_eq!(args.timeout_ms, None);

    let args: WaitArgs =
        serde_json::from_str(r#"{"timeout_ms":null}"#).expect("null should map to None");
    assert_eq!(args.timeout_ms, None);

    for bad in [
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "+12",
        "9223372036854775808",
    ] {
        let err = serde_json::from_str::<WaitArgs>(&format!(r#"{{"timeout_ms":"{bad}"}}"#))
            .expect_err("malformed integer string should be rejected");
        assert!(
            err.to_string().contains(EXPECTED_INTEGER_ERROR),
            "input {bad:?} should carry the pinned integer error, got: {err}"
        );
    }

    for bad in ["1.5", "true", "[7500]", r#"{"ms":7500}"#] {
        assert!(
            serde_json::from_str::<WaitArgs>(&format!(r#"{{"timeout_ms":{bad}}}"#)).is_err(),
            "non-integer JSON value {bad} should be rejected"
        );
    }
}
