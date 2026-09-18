use super::*;
use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::json;

/// Exact §3.2 error text for strings that are not integer literals. Duplicated
/// here instead of importing the module's constant so the pin cannot drift
/// along with a renamed implementation.
const EXPECTED_INTEGER_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

#[derive(Debug, Deserialize)]
struct U64Case {
    #[serde(default, deserialize_with = "strict_u64")]
    value: u64,

    #[serde(default, deserialize_with = "strict_u64_opt")]
    opt: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UsizeCase {
    #[serde(default, deserialize_with = "strict_usize")]
    value: usize,

    #[serde(default, deserialize_with = "strict_usize_opt")]
    opt: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct I32Case {
    #[serde(default, deserialize_with = "strict_i32")]
    value: i32,

    #[serde(default, deserialize_with = "strict_i32_opt")]
    opt: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct I64Case {
    #[serde(default, deserialize_with = "strict_i64")]
    value: i64,

    #[serde(default, deserialize_with = "strict_i64_opt")]
    opt: Option<i64>,
}

fn plain_u64(token: serde_json::Value) -> Result<u64, serde_json::Error> {
    serde_json::from_value::<U64Case>(json!({ "value": token })).map(|case| case.value)
}

fn opt_u64(token: serde_json::Value) -> Result<Option<u64>, serde_json::Error> {
    serde_json::from_value::<U64Case>(json!({ "opt": token })).map(|case| case.opt)
}

fn plain_usize(token: serde_json::Value) -> Result<usize, serde_json::Error> {
    serde_json::from_value::<UsizeCase>(json!({ "value": token })).map(|case| case.value)
}

fn opt_usize(token: serde_json::Value) -> Result<Option<usize>, serde_json::Error> {
    serde_json::from_value::<UsizeCase>(json!({ "opt": token })).map(|case| case.opt)
}

fn plain_i32(token: serde_json::Value) -> Result<i32, serde_json::Error> {
    serde_json::from_value::<I32Case>(json!({ "value": token })).map(|case| case.value)
}

fn opt_i32(token: serde_json::Value) -> Result<Option<i32>, serde_json::Error> {
    serde_json::from_value::<I32Case>(json!({ "opt": token })).map(|case| case.opt)
}

fn plain_i64(token: serde_json::Value) -> Result<i64, serde_json::Error> {
    serde_json::from_value::<I64Case>(json!({ "value": token })).map(|case| case.value)
}

fn opt_i64(token: serde_json::Value) -> Result<Option<i64>, serde_json::Error> {
    serde_json::from_value::<I64Case>(json!({ "opt": token })).map(|case| case.opt)
}

/// Assert the exact error text (snapshot pin; `from_value` errors carry no
/// position suffix).
fn expect_error(expected: &str, actual: serde_json::Error) {
    assert_eq!(actual.to_string(), expected);
}

// ---------------------------------------------------------------------------
// u64
// ---------------------------------------------------------------------------

#[test]
fn strict_u64_accepts_in_range_numbers() {
    assert_eq!(plain_u64(json!(0)).unwrap(), 0);
    assert_eq!(plain_u64(json!(42)).unwrap(), 42);
    assert_eq!(plain_u64(json!(u64::MAX)).unwrap(), u64::MAX);
}

#[test]
fn strict_u64_rejects_out_of_range_numbers_with_pinned_errors() {
    expect_error(
        "invalid value: integer `-1`, expected u64",
        plain_u64(json!(-1)).unwrap_err(),
    );
}

#[test]
fn strict_u64_rejects_float_bool_null_with_pinned_errors() {
    expect_error(
        "invalid type: floating point `1.5`, expected u64",
        plain_u64(json!(1.5)).unwrap_err(),
    );
    expect_error(
        "invalid type: boolean `true`, expected u64",
        plain_u64(json!(true)).unwrap_err(),
    );
    expect_error(
        "invalid type: unit value, expected u64",
        plain_u64(json!(null)).unwrap_err(),
    );
}

#[test]
fn strict_u64_rejects_sequence_and_map_roots_with_pinned_errors() {
    expect_error(
        "invalid type: sequence, expected u64",
        plain_u64(json!([1, 2])).unwrap_err(),
    );
    expect_error(
        "invalid type: map, expected u64",
        plain_u64(json!({})).unwrap_err(),
    );
}

#[test]
fn strict_u64_coerces_integer_literal_strings() {
    assert_eq!(plain_u64(json!("120000")).unwrap(), 120000);
    assert_eq!(plain_u64(json!("0123")).unwrap(), 123);
}

#[test]
fn strict_u64_rejects_non_literal_strings_with_pinned_errors() {
    for token in [
        "-5",
        "+12",
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "0x10",
        "99999999999999999999",
    ] {
        expect_error(EXPECTED_INTEGER_ERROR, plain_u64(json!(token)).unwrap_err());
    }
}

#[test]
fn strict_u64_opt_null_becomes_none() {
    assert_eq!(opt_u64(json!(null)).unwrap(), None);
}

#[test]
fn strict_u64_opt_coerces_numbers_and_digit_strings() {
    assert_eq!(opt_u64(json!(42)).unwrap(), Some(42));
    assert_eq!(opt_u64(json!("120000")).unwrap(), Some(120000));
    assert_eq!(opt_u64(json!("0123")).unwrap(), Some(123));
}

#[test]
fn strict_u64_opt_rejects_bad_values_with_pinned_errors() {
    expect_error(EXPECTED_INTEGER_ERROR, opt_u64(json!("abc")).unwrap_err());
    expect_error(
        "invalid value: integer `-1`, expected u64",
        opt_u64(json!(-1)).unwrap_err(),
    );
}

// ---------------------------------------------------------------------------
// usize
// ---------------------------------------------------------------------------

#[test]
fn strict_usize_accepts_in_range_numbers() {
    assert_eq!(plain_usize(json!(0)).unwrap(), 0);
    assert_eq!(plain_usize(json!(42)).unwrap(), 42);
}

#[test]
fn strict_usize_rejects_out_of_range_numbers_with_pinned_errors() {
    expect_error(
        "invalid value: integer `-1`, expected usize",
        plain_usize(json!(-1)).unwrap_err(),
    );
}

#[test]
fn strict_usize_rejects_float_bool_null_with_pinned_errors() {
    expect_error(
        "invalid type: floating point `1.5`, expected usize",
        plain_usize(json!(1.5)).unwrap_err(),
    );
    expect_error(
        "invalid type: boolean `true`, expected usize",
        plain_usize(json!(true)).unwrap_err(),
    );
    expect_error(
        "invalid type: unit value, expected usize",
        plain_usize(json!(null)).unwrap_err(),
    );
}

#[test]
fn strict_usize_rejects_sequence_and_map_roots_with_pinned_errors() {
    expect_error(
        "invalid type: sequence, expected usize",
        plain_usize(json!([1, 2])).unwrap_err(),
    );
    expect_error(
        "invalid type: map, expected usize",
        plain_usize(json!({})).unwrap_err(),
    );
}

#[test]
fn strict_usize_coerces_integer_literal_strings() {
    assert_eq!(plain_usize(json!("120000")).unwrap(), 120000);
    assert_eq!(plain_usize(json!("0123")).unwrap(), 123);
}

#[test]
fn strict_usize_rejects_non_literal_strings_with_pinned_errors() {
    for token in [
        "-5",
        "+12",
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "0x10",
        "99999999999999999999",
    ] {
        expect_error(
            EXPECTED_INTEGER_ERROR,
            plain_usize(json!(token)).unwrap_err(),
        );
    }
}

#[test]
fn strict_usize_opt_null_becomes_none() {
    assert_eq!(opt_usize(json!(null)).unwrap(), None);
}

#[test]
fn strict_usize_opt_coerces_numbers_and_digit_strings() {
    assert_eq!(opt_usize(json!(42)).unwrap(), Some(42));
    assert_eq!(opt_usize(json!("120000")).unwrap(), Some(120000));
    assert_eq!(opt_usize(json!("0123")).unwrap(), Some(123));
}

#[test]
fn strict_usize_opt_rejects_bad_values_with_pinned_errors() {
    expect_error(EXPECTED_INTEGER_ERROR, opt_usize(json!("abc")).unwrap_err());
    expect_error(
        "invalid value: integer `-1`, expected usize",
        opt_usize(json!(-1)).unwrap_err(),
    );
}

// ---------------------------------------------------------------------------
// i32
// ---------------------------------------------------------------------------

#[test]
fn strict_i32_accepts_in_range_numbers() {
    assert_eq!(plain_i32(json!(i32::MIN)).unwrap(), i32::MIN);
    assert_eq!(plain_i32(json!(-5)).unwrap(), -5);
    assert_eq!(plain_i32(json!(0)).unwrap(), 0);
    assert_eq!(plain_i32(json!(42)).unwrap(), 42);
    assert_eq!(plain_i32(json!(i32::MAX)).unwrap(), i32::MAX);
}

#[test]
fn strict_i32_rejects_out_of_range_numbers_with_pinned_errors() {
    expect_error(
        "invalid value: integer `2147483648`, expected i32",
        plain_i32(json!(i32::MAX as i64 + 1)).unwrap_err(),
    );
    expect_error(
        "invalid value: integer `-2147483649`, expected i32",
        plain_i32(json!(i32::MIN as i64 - 1)).unwrap_err(),
    );
}

#[test]
fn strict_i32_rejects_float_bool_null_with_pinned_errors() {
    expect_error(
        "invalid type: floating point `1.5`, expected i32",
        plain_i32(json!(1.5)).unwrap_err(),
    );
    expect_error(
        "invalid type: boolean `true`, expected i32",
        plain_i32(json!(true)).unwrap_err(),
    );
    expect_error(
        "invalid type: unit value, expected i32",
        plain_i32(json!(null)).unwrap_err(),
    );
}

#[test]
fn strict_i32_rejects_sequence_and_map_roots_with_pinned_errors() {
    expect_error(
        "invalid type: sequence, expected i32",
        plain_i32(json!([1, 2])).unwrap_err(),
    );
    expect_error(
        "invalid type: map, expected i32",
        plain_i32(json!({})).unwrap_err(),
    );
}

#[test]
fn strict_i32_coerces_integer_literal_strings() {
    assert_eq!(plain_i32(json!("120000")).unwrap(), 120000);
    assert_eq!(plain_i32(json!("0123")).unwrap(), 123);
    assert_eq!(plain_i32(json!("-5")).unwrap(), -5);
}

#[test]
fn strict_i32_rejects_non_literal_strings_with_pinned_errors() {
    for token in [
        "+12",
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "0x10",
        "99999999999999999999",
    ] {
        expect_error(EXPECTED_INTEGER_ERROR, plain_i32(json!(token)).unwrap_err());
    }
}

#[test]
fn strict_i32_opt_null_becomes_none() {
    assert_eq!(opt_i32(json!(null)).unwrap(), None);
}

#[test]
fn strict_i32_opt_coerces_numbers_and_digit_strings() {
    assert_eq!(opt_i32(json!(42)).unwrap(), Some(42));
    assert_eq!(opt_i32(json!("120000")).unwrap(), Some(120000));
    assert_eq!(opt_i32(json!("-5")).unwrap(), Some(-5));
}

#[test]
fn strict_i32_opt_rejects_bad_values_with_pinned_errors() {
    expect_error(EXPECTED_INTEGER_ERROR, opt_i32(json!("abc")).unwrap_err());
    expect_error(
        "invalid value: integer `2147483648`, expected i32",
        opt_i32(json!(i32::MAX as i64 + 1)).unwrap_err(),
    );
}

// ---------------------------------------------------------------------------
// i64
// ---------------------------------------------------------------------------

#[test]
fn strict_i64_accepts_in_range_numbers() {
    assert_eq!(plain_i64(json!(i64::MIN)).unwrap(), i64::MIN);
    assert_eq!(plain_i64(json!(-5)).unwrap(), -5);
    assert_eq!(plain_i64(json!(0)).unwrap(), 0);
    assert_eq!(plain_i64(json!(42)).unwrap(), 42);
    assert_eq!(plain_i64(json!(i64::MAX)).unwrap(), i64::MAX);
}

#[test]
fn strict_i64_rejects_out_of_range_numbers_with_pinned_errors() {
    expect_error(
        "invalid value: integer `9223372036854775808`, expected i64",
        plain_i64(json!(i64::MAX as u64 + 1)).unwrap_err(),
    );
}

#[test]
fn strict_i64_rejects_float_bool_null_with_pinned_errors() {
    expect_error(
        "invalid type: floating point `1.5`, expected i64",
        plain_i64(json!(1.5)).unwrap_err(),
    );
    expect_error(
        "invalid type: boolean `true`, expected i64",
        plain_i64(json!(true)).unwrap_err(),
    );
    expect_error(
        "invalid type: unit value, expected i64",
        plain_i64(json!(null)).unwrap_err(),
    );
}

#[test]
fn strict_i64_rejects_sequence_and_map_roots_with_pinned_errors() {
    expect_error(
        "invalid type: sequence, expected i64",
        plain_i64(json!([1, 2])).unwrap_err(),
    );
    expect_error(
        "invalid type: map, expected i64",
        plain_i64(json!({})).unwrap_err(),
    );
}

#[test]
fn strict_i64_coerces_integer_literal_strings() {
    assert_eq!(plain_i64(json!("120000")).unwrap(), 120000);
    assert_eq!(plain_i64(json!("0123")).unwrap(), 123);
    assert_eq!(plain_i64(json!("-5")).unwrap(), -5);
}

#[test]
fn strict_i64_rejects_non_literal_strings_with_pinned_errors() {
    for token in [
        "+12",
        "1.5",
        "12abc",
        " 12",
        "",
        "null",
        "0x10",
        "99999999999999999999",
    ] {
        expect_error(EXPECTED_INTEGER_ERROR, plain_i64(json!(token)).unwrap_err());
    }
}

#[test]
fn strict_i64_opt_null_becomes_none() {
    assert_eq!(opt_i64(json!(null)).unwrap(), None);
}

#[test]
fn strict_i64_opt_coerces_numbers_and_digit_strings() {
    assert_eq!(opt_i64(json!(42)).unwrap(), Some(42));
    assert_eq!(opt_i64(json!("120000")).unwrap(), Some(120000));
    assert_eq!(opt_i64(json!("-5")).unwrap(), Some(-5));
}

#[test]
fn strict_i64_opt_rejects_bad_values_with_pinned_errors() {
    expect_error(EXPECTED_INTEGER_ERROR, opt_i64(json!("abc")).unwrap_err());
    expect_error(
        "invalid value: integer `9223372036854775808`, expected i64",
        opt_i64(json!(i64::MAX as u64 + 1)).unwrap_err(),
    );
}
