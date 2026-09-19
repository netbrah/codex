//! Shared tool-argument parse funnel (Ratchet B, apex-xt2.7).
//!
//! Every handler parse of raw model-supplied JSON arguments funnels through
//! `parse_arguments` so a parse failure names the tool and, best-effort,
//! the parameter the model should fix. Message shapes are pinned by
//! `docs/tool-args-ratchet-spec.md` §4.2: the raw serde error is always
//! appended verbatim.

use serde::Deserialize;

use crate::function_tool::FunctionCallError;

pub(crate) fn parse_arguments<T>(tool_name: &str, arguments: &str) -> Result<T, FunctionCallError>
where
    T: for<'de> Deserialize<'de>,
{
    match serde_json::from_str(arguments) {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            let base = format!("failed to parse arguments for {tool_name}: {err}");
            let message = if err.classify() == serde_json::error::Category::Eof {
                format!("{base} — arguments appear truncated; resend the complete JSON object")
            } else {
                match nearest_json_key_before(arguments, err.line(), err.column()) {
                    Some(parameter) => format!("{base} (parameter \"{parameter}\")"),
                    None => base,
                }
            };
            Err(FunctionCallError::RespondToModel(message))
        }
    }
}

/// Best-effort parameter hint for a serde position: the nearest object key
/// whose `:` sits at or before one byte past the reported (line, column).
///
/// serde_json reports `column()` as a 1-based byte offset within the line
/// (vendored serde_json 1.0.149: `iter.rs` counts bytes, `error.rs` keeps
/// 1-based columns). A type error on a string value lands after the closing
/// quote; an unknown-field error lands on the key's closing quote with the
/// `:` one byte past — hence the inclusive `offset + 1` boundary. A
/// wrong-but-adjacent hint is acceptable by contract; a missing hint is
/// preferred over a confidently wrong one.
pub(crate) fn nearest_json_key_before(json: &str, line: usize, column: usize) -> Option<String> {
    if line == 0 || column == 0 {
        return None;
    }
    let bytes = json.as_bytes();
    let line_count = bytes.iter().filter(|byte| **byte == b'\n').count() + 1;
    if line > line_count {
        return None;
    }
    let mut line_start = 0usize;
    let mut newlines_seen = 0usize;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            newlines_seen += 1;
            if newlines_seen == line - 1 {
                line_start = index + 1;
                break;
            }
        }
    }
    let offset = (line_start + column - 1).min(bytes.len());
    scan_for_key(bytes, offset)
}

/// Scans backward from the reported position for the nearest `"..."` token
/// that is an object key: an unescaped string followed, after optional
/// whitespace, by `:` at or before `offset + 1` (0-based, inclusive).
fn scan_for_key(bytes: &[u8], offset: usize) -> Option<String> {
    let mut end = (offset + 1).min(bytes.len());
    while end > 0 {
        let close = bytes[..end].iter().rposition(|byte| *byte == b'"')?;
        if let Some((open, colon_index)) = key_token_position(bytes, close)
            && colon_index <= offset + 1
        {
            if let Ok(key) = std::str::from_utf8(&bytes[open + 1..close]) {
                return Some(key.to_string());
            }
        }
        end = close;
    }
    None
}

/// If the quote at `close` ends a `"..."` string token, returns its opening
/// quote and the index of the `:` that follows the token after optional
/// whitespace.
fn key_token_position(bytes: &[u8], close: usize) -> Option<(usize, usize)> {
    let open = find_opening_quote(bytes, close)?;
    let mut index = close + 1;
    while index < bytes.len() && matches!(bytes[index], b' ' | b'\t' | b'\n' | b'\r') {
        index += 1;
    }
    (index < bytes.len() && bytes[index] == b':').then_some((open, index))
}

/// The previous unescaped `"` before `close` (an escaped `\"` is content,
/// not a token boundary).
fn find_opening_quote(bytes: &[u8], close: usize) -> Option<usize> {
    for (index, byte) in bytes[..close].iter().enumerate().rev() {
        if *byte != b'"' {
            continue;
        }
        let mut backslashes = 0usize;
        let mut probe = index;
        while probe > 0 && bytes[probe - 1] == b'\\' {
            backslashes += 1;
            probe -= 1;
        }
        if backslashes % 2 == 0 {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
#[path = "args_parse_tests.rs"]
mod tests;
