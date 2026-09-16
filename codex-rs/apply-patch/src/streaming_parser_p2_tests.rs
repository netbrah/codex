//! P2 Add-File leniency tests (spec §4 T1 sub-cases;
//! `docs/responses-compat-apply-patch-format.md` §3.2).

use pretty_assertions::assert_eq;
use std::path::PathBuf;

use super::*;

#[test]
fn test_p2_add_file_raw_markdown_f1_replay_matches_canonical() {
    // T1.1: an Add-File with raw markdown content (`# H1` first line, tables,
    // blank lines) parses; `contents` is byte-identical to the canonical
    // `+`-prefixed version of the same file.
    let content =
        "# H1\n\n| name | value |\n| ---- | ----- |\n| a    | 1     |\n\nsome **bold** text\n";
    let raw_patch =
        format!("*** Begin Patch\n*** Add File: docs/readme.md\n{content}*** End Patch\n");
    let prefixed_body = content
        .lines()
        .map(|line| format!("+{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let canonical_patch =
        format!("*** Begin Patch\n*** Add File: docs/readme.md\n{prefixed_body}\n*** End Patch\n");

    let mut raw_parser = StreamingPatchParser::default();
    let raw_hunks = raw_parser.push_delta(&raw_patch).unwrap();
    let mut canonical_parser = StreamingPatchParser::default();
    let canonical_hunks = canonical_parser.push_delta(&canonical_patch).unwrap();

    assert_eq!(raw_hunks, canonical_hunks);
    assert_eq!(
        raw_hunks,
        vec![AddFile {
            path: PathBuf::from("docs/readme.md"),
            contents: content.to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_empty_line_becomes_empty_content_line() {
    // T1.2: an empty line inside Add-File content becomes an empty line in
    // `contents` (newly accepted; was an error before P2).
    let patch = "*** Begin Patch\n*** Add File: f.txt\na\n\nb\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "a\n\nb\n".to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_mixed_prefixed_and_raw_lines_concatenate() {
    // T1.3: mixed `+`-prefixed and raw lines concatenate in order
    // (`+` lines strip the prefix, raw lines are verbatim).
    let patch = "*** Begin Patch\n*** Add File: f.txt\n+hello\nworld\n+there\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "hello\nworld\nthere\n".to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_raw_stars_line_matching_no_marker_is_content() {
    // T1.7: a raw content line beginning `*** ` but matching no known marker
    // is appended verbatim as content.
    let patch = "*** Begin Patch\n*** Add File: f.txt\n*** Note: keep this line\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "*** Note: keep this line\n".to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_real_headers_inside_add_file_remain_structural() {
    // T1.7: each real header line inside Add-File content keeps its structural
    // meaning (next `*** Add File:`, `*** Delete File:`, `*** Update File:`,
    // `*** End Patch`) — handled before the content branch.
    let patch = "*** Begin Patch\n*** Add File: a.txt\nx\n*** Add File: b.txt\ny\n*** Delete File: d.txt\n*** Update File: u.txt\n@@\n z\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![
            AddFile {
                path: PathBuf::from("a.txt"),
                contents: "x\n".to_string(),
            },
            AddFile {
                path: PathBuf::from("b.txt"),
                contents: "y\n".to_string(),
            },
            DeleteFile {
                path: PathBuf::from("d.txt"),
            },
            UpdateFile {
                path: PathBuf::from("u.txt"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: vec!["z".to_string()],
                    new_lines: vec!["z".to_string()],
                    context_line_indices: vec![(0, 0)],
                    is_end_of_file: false,
                }],
            },
        ]
    );
}

#[test]
fn test_p2_add_file_whitespace_padded_end_patch_is_structural() {
    // T1.7: `*** End Patch` with surrounding whitespace is still structural
    // (trimmed match, pre-existing) and terminates the patch.
    let patch = "*** Begin Patch\n*** Add File: f.txt\nx\n   *** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "x\n".to_string(),
        }]
    );
    assert_eq!(
        parser.finish(),
        Ok(vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "x\n".to_string(),
        }])
    );
}
#[test]
fn test_p2_add_file_typoed_structural_line_swallowed_as_content() {
    // T1.8: a typo'd structural line inside Add-File (`*** Ad File: x`) is
    // swallowed as content instead of erroring, and the rest of the patch
    // applies (pins the spec §3.2 silent-partial-apply class).
    let patch =
        "*** Begin Patch\n*** Add File: a.txt\n*** Ad File: x\nreal content\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("a.txt"),
            contents: "*** Ad File: x\nreal content\n".to_string(),
        }]
    );
    assert_eq!(
        parser.finish(),
        Ok(vec![AddFile {
            path: PathBuf::from("a.txt"),
            contents: "*** Ad File: x\nreal content\n".to_string(),
        }])
    );
}
#[test]
fn test_p2_add_file_raw_plus_prefixed_line_strips_leading_plus() {
    // T1.9 (named lossy case): a raw content line that starts with `+` is
    // parsed as a prefixed line — its leading `+` is stripped (identical to
    // today's canonical behavior for `+` lines).
    let patch = "*** Begin Patch\n*** Add File: f.txt\n++42 20 7946 0958\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "+42 20 7946 0958\n".to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_to_existing_path_parses_without_existence_check() {
    // T1.9 (spec §3.2/§3.4 decision): no existence check on any path. At the
    // parser level a raw Add-File parses as-is into an AddFile hunk; the
    // overwrite outcome for an existing file is the apply layer's and is
    // pinned (unmodified) by fixture scenario 011_add_overwrites_existing_file
    // and the core-suite test apply_patch_cli_add_overwrites_existing_file.
    let patch = "*** Begin Patch\n*** Add File: existing/file.txt\nraw line one\nraw line two\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("existing/file.txt"),
            contents: "raw line one\nraw line two\n".to_string(),
        }]
    );
}
#[test]
fn test_p2_add_file_last_line_canonical_end_patch_is_content() {
    // T1.9b (canonical variant): the canonical line `+*** End Patch` is
    // preserved verbatim as content — the structural check runs on the
    // trimmed line and the `+` branch takes the raw line, so the taught form
    // is complete (spec §3.2 probe: contents `line1\n*** End Patch\n`).
    let patch = "*** Begin Patch\n*** Add File: f.txt\nline1\n+*** End Patch\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "line1\n*** End Patch\n".to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_last_line_raw_end_patch_truncates() {
    // T1.9b (raw variant): an unprefixed final content line exactly
    // `*** End Patch` is truncated at that line (structural check runs before
    // content branches) — a lenient-form-only loss pinned by spec §3.2.
    let patch = "*** Begin Patch\n*** Add File: f.txt\nline1\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "line1\n".to_string(),
        }]
    );
}
#[test]
fn test_p2_add_file_whitespace_only_line_is_whitespace_content() {
    // T1.10: a whitespace-only line becomes that whitespace as content
    // (verbatim; trimming would corrupt indentation).
    let patch = "*** Begin Patch\n*** Add File: f.txt\n   \n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "   \n".to_string(),
        }]
    );
}

#[test]
fn test_p2_add_file_environment_id_line_is_content() {
    // T1.10: `*** Environment ID:` inside Add-File content is content (the
    // env-id is honored only in `StartedPatch`, pre-existing); `environment_id()`
    // stays unset.
    let patch = "*** Begin Patch\n*** Add File: f.txt\n*** Environment ID: env1\n*** End Patch\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "*** Environment ID: env1\n".to_string(),
        }]
    );
    assert_eq!(parser.environment_id(), None);
}

#[test]
fn test_p2_add_file_unclosed_patch_finish_error_unchanged() {
    // T1.10: an unclosed patch keeps the unchanged `finish()`/boundary error.
    let patch = "*** Begin Patch\n*** Add File: f.txt\nraw";
    let mut parser = StreamingPatchParser::default();
    parser.push_delta(patch).unwrap();

    assert_eq!(
        parser.finish(),
        Err(InvalidPatchError(
            "The last line of the patch must be '*** End Patch'".to_string()
        ))
    );
}

#[test]
fn test_p2_add_file_crlf_raw_lines_are_lf_equivalent() {
    // T1.10: a CRLF raw Add-File is LF-equivalent (one trailing `\r` stripped
    // per line by `push_delta` before `process_line`, unchanged path).
    let patch = "*** Begin Patch\r\n*** Add File: f.txt\r\nraw line\r\n*** End Patch\r\n";
    let mut parser = StreamingPatchParser::default();
    let hunks = parser.push_delta(patch).unwrap();

    assert_eq!(
        hunks,
        vec![AddFile {
            path: PathBuf::from("f.txt"),
            contents: "raw line\n".to_string(),
        }]
    );
}
#[test]
fn test_p2_update_file_raw_unprefixed_lines_still_rejected() {
    // T1.11 (regression lock): Update-File with raw unprefixed lines is still
    // rejected with the unchanged message — P2 leniency is Add-File only
    // (spec §3.4: no Update-File leniency).
    let patch = "*** Begin Patch\n*** Update File: file.txt\nbad\n";
    let mut parser = StreamingPatchParser::default();

    assert_eq!(
        parser.push_delta(patch),
        Err(InvalidHunkError {
            message: "Unexpected line found in update hunk: 'bad'. Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)"
                .to_string(),
            line_number: 3,
        })
    );
}
