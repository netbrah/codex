use super::*;
use pretty_assertions::assert_eq;

// Byte-identical copy of the private `expected an integer` message owned by
// codex-tools `strict_int` (see `strict_int_tests.rs` there). Drift pair:
// change both when one changes.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

#[test]
fn exec_wait_args_coerces_string_integers() {
    let args: ExecWaitArgs =
        serde_json::from_str(r#"{"cell_id":"cell-1","yield_time_ms":"100","max_tokens":"500"}"#)
            .expect("string integer literals should parse");
    assert_eq!(args.yield_time_ms, 100);
    assert_eq!(args.max_tokens, Some(500));

    let args: ExecWaitArgs = serde_json::from_str(r#"{"cell_id":"cell-1","yield_time_ms":"0100"}"#)
        .expect("leading zeros should parse");
    assert_eq!(args.yield_time_ms, 100);

    let args: ExecWaitArgs =
        serde_json::from_str(r#"{"cell_id":"cell-1","yield_time_ms":250,"max_tokens":1024}"#)
            .expect("JSON numbers should still parse");
    assert_eq!(args.yield_time_ms, 250);
    assert_eq!(args.max_tokens, Some(1024));

    let args: ExecWaitArgs = serde_json::from_str(r#"{"cell_id":"cell-1","max_tokens":null}"#)
        .expect("null should map to None");
    assert_eq!(args.max_tokens, None);

    let err = serde_json::from_str::<ExecWaitArgs>(r#"{"cell_id":"cell-1","yield_time_ms":"-5"}"#)
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
        let err = serde_json::from_str::<ExecWaitArgs>(&format!(
            r#"{{"cell_id":"cell-1","yield_time_ms":"{bad}"}}"#
        ))
        .expect_err("malformed integer string should be rejected");
        assert!(
            err.to_string().contains(EXPECTED_INTEGER_ERROR),
            "input {bad:?} should carry the pinned integer error, got: {err}"
        );
    }

    for bad in ["1.5", "true", "[250]", r#"{"ms":250}"#] {
        assert!(
            serde_json::from_str::<ExecWaitArgs>(&format!(
                r#"{{"cell_id":"cell-1","yield_time_ms":{bad}}}"#
            ))
            .is_err(),
            "non-integer JSON value {bad} should be rejected"
        );
    }
}

#[test]
fn exec_wait_args_rejects_unknown_field() {
    let err = serde_json::from_str::<ExecWaitArgs>(r#"{"cell_id":"cell-1","bogus":1}"#)
        .expect_err("unknown fields must be rejected");
    assert!(
        err.to_string().contains("unknown field `bogus`"),
        "unknown field error should name the field, got: {err}"
    );
}
