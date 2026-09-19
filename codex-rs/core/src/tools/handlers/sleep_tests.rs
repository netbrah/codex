use super::*;
use pretty_assertions::assert_eq;

// Byte-identical copy of the private `expected an integer` message owned by
// codex-tools `strict_int` (see `strict_int_tests.rs` there). Drift pair:
// change both when one changes.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

#[test]
fn sleep_args_coerces_string_integers() {
    let args: SleepArgs = serde_json::from_str(r#"{"duration_ms":"1500"}"#)
        .expect("string integer literals should parse");
    assert_eq!(args.duration_ms, 1500);

    let args: SleepArgs =
        serde_json::from_str(r#"{"duration_ms":"0123"}"#).expect("leading zeros should parse");
    assert_eq!(args.duration_ms, 123);

    let args: SleepArgs =
        serde_json::from_str(r#"{"duration_ms":2500}"#).expect("JSON numbers should still parse");
    assert_eq!(args.duration_ms, 2500);

    let err = serde_json::from_str::<SleepArgs>(r#"{"duration_ms":"-5"}"#)
        .expect_err("negative strings must be rejected for unsigned fields");
    assert!(
        err.to_string().contains(EXPECTED_INTEGER_ERROR),
        "negative unsigned input should carry the pinned integer error, got: {err}"
    );

    for bad in [
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "+12",
        "18446744073709551616",
    ] {
        let err = serde_json::from_str::<SleepArgs>(&format!(r#"{{"duration_ms":"{bad}"}}"#))
            .expect_err("malformed integer string should be rejected");
        assert!(
            err.to_string().contains(EXPECTED_INTEGER_ERROR),
            "input {bad:?} should carry the pinned integer error, got: {err}"
        );
    }

    for bad in ["1.5", "true", "[2500]", r#"{"ms":2500}"#] {
        assert!(
            serde_json::from_str::<SleepArgs>(&format!(r#"{{"duration_ms":{bad}}}"#)).is_err(),
            "non-integer JSON value {bad} should be rejected"
        );
    }
}
