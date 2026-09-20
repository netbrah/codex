//! Unit tests for the shared tool-argument parse funnel (Ratchet B).
//!
//! Message shapes mirror `docs/tool-args-ratchet-spec.md` §4.2: the raw
//! serde error is always appended verbatim, and `nearest_json_key_before`
//! is best-effort (a missing hint is preferred over a confidently wrong one).

use pretty_assertions::assert_eq;
use serde::Deserialize;

use crate::function_tool::FunctionCallError;
use crate::tools::handlers::args_parse::nearest_json_key_before;
use crate::tools::handlers::parse_arguments;

#[derive(Deserialize)]
struct TimeoutArgs {
    timeout_ms: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictTimeoutArgs {
    #[allow(dead_code)]
    timeout_ms: i64,
}

#[derive(Deserialize)]
struct PairArgs {
    #[allow(dead_code)]
    first: i64,
    #[allow(dead_code)]
    second: i64,
}

#[derive(Deserialize)]
struct ElementArgs(u32);

fn parse_message<T>(tool_name: &str, arguments: &str) -> String
where
    T: for<'de> Deserialize<'de>,
{
    match parse_arguments::<T>(tool_name, arguments) {
        Ok(_) => panic!("expected a parse error for {arguments:?}"),
        Err(FunctionCallError::RespondToModel(message)) => message,
        Err(err) => panic!("expected RespondToModel error, got: {err:?}"),
    }
}

#[test]
fn parse_arguments_happy_path_keeps_parsed_value() {
    let args: TimeoutArgs =
        parse_arguments("wait_agent", r#"{"timeout_ms":5}"#).expect("valid arguments should parse");
    assert_eq!(args.timeout_ms, 5);
}

#[test]
fn shape1_type_error_names_tool_and_parameter() {
    let message = parse_message::<TimeoutArgs>("wait_agent", r#"{"timeout_ms":"abc"}"#);
    assert_eq!(
        message,
        "failed to parse arguments for wait_agent: invalid type: string \"abc\", expected i64 at line 1 column 19 (parameter \"timeout_ms\")"
    );
}

#[test]
fn shape1_unknown_field_names_tool_and_parameter() {
    let message = parse_message::<StrictTimeoutArgs>("wait_agent", r#"{"target":"task_1"}"#);
    assert_eq!(
        message,
        "failed to parse arguments for wait_agent: unknown field `target`, expected `timeout_ms` at line 1 column 9 (parameter \"target\")"
    );
}

#[test]
fn shape1_missing_field_names_tool_and_last_key() {
    let message = parse_message::<TimeoutArgs>("wait_agent", r#"{"target":"task_1"}"#);
    assert_eq!(
        message,
        "failed to parse arguments for wait_agent: missing field `timeout_ms` at line 1 column 19 (parameter \"target\")"
    );
}

#[test]
fn shape1_key_after_error_position_is_not_attributed() {
    let message = parse_message::<PairArgs>("wait_agent", r#"{"first":"bad","second":1}"#);
    assert_eq!(
        message,
        "failed to parse arguments for wait_agent: invalid type: string \"bad\", expected i64 at line 1 column 14 (parameter \"first\")"
    );
}

#[test]
fn shape1_no_key_before_error_gives_no_hint() {
    let message = parse_message::<ElementArgs>("wait_agent", r#"["x"]"#);
    assert_eq!(
        message,
        "failed to parse arguments for wait_agent: invalid type: sequence, expected u32 at line 1 column 0"
    );
}

#[test]
fn shape2_truncated_input_gets_eof_hint_without_parameter() {
    let message = parse_message::<TimeoutArgs>("exec_command", r#"{"cmd": "ls","#);
    assert_eq!(
        message,
        "failed to parse arguments for exec_command: EOF while parsing a value at line 1 column 13 — arguments appear truncated; resend the complete JSON object"
    );
}

#[test]
fn shape3_non_object_root_is_plain_error() {
    let message = parse_message::<TimeoutArgs>("wait_agent", "[1,2]");
    assert_eq!(
        message,
        "failed to parse arguments for wait_agent: trailing characters at line 1 column 4"
    );
}

#[test]
fn nearest_key_nested_object_inner_key_wins() {
    assert_eq!(
        nearest_json_key_before(r#"{"outer":{"inner":1}}"#, 1, 20),
        Some("inner".to_string())
    );
}

#[test]
fn nearest_key_array_string_is_not_a_key() {
    assert_eq!(nearest_json_key_before(r#"["a",1]"#, 1, 7), None);
}

#[test]
fn nearest_key_uses_byte_offsets_for_unicode() {
    assert_eq!(
        nearest_json_key_before(r#"{"clé":1}"#, 1, 10),
        Some("clé".to_string())
    );
}

#[test]
fn nearest_key_unknown_field_at_colon_qualifies() {
    // Round-8 P-1 boundary: the key's closing quote ends at the reported
    // offset and the `:` sits one past it (0-based `offset + 1`, inclusive).
    assert_eq!(
        nearest_json_key_before(r#"{"target":"task_1"}"#, 1, 9),
        Some("target".to_string())
    );
}

#[test]
fn nearest_key_whitespace_before_colon_at_boundary_does_not_qualify() {
    assert_eq!(nearest_json_key_before(r#"{"a" :1}"#, 1, 4), None);
}

#[test]
fn nearest_key_peek_asymmetry_absorbed_by_backward_scan() {
    // Under vendored serde_json 1.0.149 the direct `peek_error` this input
    // raises (after the number `1`, expected `,` or `}`) reports the peeked
    // byte's own 1-based column (7 for `x`); the one-short class is the
    // `position()`/`fix_position` path — unpositioned errors such as `["x"]`
    // parsed as u32 (pinned to column 0 by
    // shape1_no_key_before_error_gives_no_hint) are located at the byte
    // before the peeked one. This test deliberately feeds the scan that
    // one-short column (6, the digit `1`) to pin the scan absorbing the
    // asymmetry and still resolving the enclosing key `a`.
    assert_eq!(
        nearest_json_key_before(r#"{"a":1x}"#, 1, 6),
        Some("a".to_string())
    );
}

#[test]
fn parse_arguments_invalid_number_still_attributes_key() {
    let message = parse_message::<TimeoutArgs>("wait_agent", r#"{"timeout_ms":1x}"#);
    assert!(message.starts_with("failed to parse arguments for wait_agent: "));
    assert!(
        message.contains("(parameter \"timeout_ms\")"),
        "invalid-number position off-by-one should still attribute the key, got: {message}"
    );
}

#[test]
fn nearest_key_empty_and_unaddressable_positions() {
    assert_eq!(nearest_json_key_before("", 1, 1), None);
    assert_eq!(nearest_json_key_before("{}", 0, 0), None);
    assert_eq!(nearest_json_key_before("{}", 9, 9), None);
}
