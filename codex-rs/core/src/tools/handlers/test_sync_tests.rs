use super::*;
use pretty_assertions::assert_eq;

// Byte-identical copy of the private `expected an integer` message owned by
// codex-tools `strict_int` (see `strict_int_tests.rs` there). Drift pair:
// change both when one changes.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

#[test]
fn test_sync_args_coerces_string_integers() {
    let args: TestSyncArgs =
        serde_json::from_str(r#"{"sleep_before_ms":"100","sleep_after_ms":"200"}"#)
            .expect("string integer literals should parse");
    assert_eq!(args.sleep_before_ms, Some(100));
    assert_eq!(args.sleep_after_ms, Some(200));

    let args: TestSyncArgs =
        serde_json::from_str(r#"{"barrier":{"id":"b1","participants":"3","timeout_ms":"5000"}}"#)
            .expect("string integer literals should parse in nested barrier args");
    let barrier = args.barrier.expect("barrier should be present");
    assert_eq!(barrier.participants, 3);
    assert_eq!(barrier.timeout_ms, 5000);

    let args: TestSyncArgs =
        serde_json::from_str(r#"{"sleep_before_ms":"0100"}"#).expect("leading zeros should parse");
    assert_eq!(args.sleep_before_ms, Some(100));

    let args: TestSyncArgs = serde_json::from_str(
        r#"{"sleep_before_ms":5,"sleep_after_ms":6,"barrier":{"id":"b2","participants":2,"timeout_ms":1000}}"#,
    )
    .expect("JSON numbers should still parse");
    assert_eq!(args.sleep_before_ms, Some(5));
    assert_eq!(args.sleep_after_ms, Some(6));
    assert_eq!(
        args.barrier
            .expect("barrier should be present")
            .participants,
        2
    );

    let args: TestSyncArgs = serde_json::from_str(r#"{}"#).expect("missing fields should parse");
    assert_eq!(args.sleep_before_ms, None);
    assert_eq!(args.sleep_after_ms, None);
    assert!(args.barrier.is_none());

    for bad in [
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "+12",
        "18446744073709551616",
    ] {
        let err =
            serde_json::from_str::<TestSyncArgs>(&format!(r#"{{"sleep_before_ms":"{bad}"}}"#))
                .expect_err("malformed integer string should be rejected");
        assert!(
            err.to_string().contains(EXPECTED_INTEGER_ERROR),
            "input {bad:?} should carry the pinned integer error, got: {err}"
        );
    }

    for bad in ["1.5", "true", "[5]", r#"{"ms":5}"#] {
        assert!(
            serde_json::from_str::<TestSyncArgs>(&format!(r#"{{"sleep_before_ms":{bad}}}"#))
                .is_err(),
            "non-integer JSON value {bad} should be rejected"
        );
    }
}

#[test]
fn barrier_args_coerces_string_integers() {
    let args: BarrierArgs =
        serde_json::from_str(r#"{"id":"b1","participants":"3","timeout_ms":"5000"}"#)
            .expect("string integer literals should parse");
    assert_eq!(args.participants, 3);
    assert_eq!(args.timeout_ms, 5000);

    let args: BarrierArgs = serde_json::from_str(r#"{"id":"b1","participants":"03"}"#)
        .expect("leading zeros should parse");
    assert_eq!(args.participants, 3);

    let args: BarrierArgs = serde_json::from_str(r#"{"id":"b1","participants":4}"#)
        .expect("JSON numbers should still parse");
    assert_eq!(args.participants, 4);

    let err = serde_json::from_str::<BarrierArgs>(r#"{"id":"b1","participants":"-5"}"#)
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
        let err = serde_json::from_str::<BarrierArgs>(&format!(
            r#"{{"id":"b1","participants":"{bad}"}}"#
        ))
        .expect_err("malformed integer string should be rejected");
        assert!(
            err.to_string().contains(EXPECTED_INTEGER_ERROR),
            "input {bad:?} should carry the pinned integer error, got: {err}"
        );
    }

    for bad in ["1.5", "true", "[4]", r#"{"n":4}"#] {
        assert!(
            serde_json::from_str::<BarrierArgs>(&format!(r#"{{"id":"b1","participants":{bad}}}"#))
                .is_err(),
            "non-integer JSON value {bad} should be rejected"
        );
    }
}
