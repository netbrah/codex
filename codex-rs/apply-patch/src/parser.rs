//! This module is responsible for parsing & validating a patch into a list of "hunks".
//! (It does not attempt to actually check that the patch can be applied to the filesystem.)
//!
//! The official Lark grammar for the apply-patch format is:
//!
//! start: begin_patch environment_id? hunk+ end_patch
//! begin_patch: "*** Begin Patch" LF
//! environment_id: "*** Environment ID: " filename LF
//! end_patch: "*** End Patch" LF?
//!
//! hunk: add_hunk | delete_hunk | update_hunk
//! add_hunk: "*** Add File: " filename LF add_line+
//! delete_hunk: "*** Delete File: " filename LF
//! update_hunk: "*** Update File: " filename LF change_move? change?
//! filename: /(.+)/
//! add_line: "+" /(.+)/ LF -> line
//!
//! change_move: "*** Move to: " filename LF
//! change: (change_context | change_line)+ eof_line?
//! change_context: ("@@" | "@@ " /(.+)/) LF
//! change_line: ("+" | "-" | " ") /(.+)/ LF
//! eof_line: "*** End of File" LF
//!
//! The parser below is a little more lenient than the explicit spec and allows for
//! leading/trailing whitespace around patch markers.
//! The streaming parser's `AddFile` arm additionally accepts non-`+`-prefixed content lines verbatim (lenient add-file content); the grammar listing above remains the canonical form.
use crate::ApplyPatchArgs;
use crate::streaming_parser::StreamingPatchParser;
#[cfg(test)]
use codex_utils_absolute_path::test_support::PathBufExt;
use codex_utils_path_uri::PathUri;
use codex_utils_path_uri::PathUriParseError;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;

pub(crate) const BEGIN_PATCH_MARKER: &str = "*** Begin Patch";
pub(crate) const END_PATCH_MARKER: &str = "*** End Patch";
pub(crate) const ADD_FILE_MARKER: &str = "*** Add File: ";
pub(crate) const DELETE_FILE_MARKER: &str = "*** Delete File: ";
pub(crate) const UPDATE_FILE_MARKER: &str = "*** Update File: ";
pub(crate) const MOVE_TO_MARKER: &str = "*** Move to: ";
pub(crate) const EOF_MARKER: &str = "*** End of File";
pub(crate) const CHANGE_CONTEXT_MARKER: &str = "@@ ";
pub(crate) const EMPTY_CHANGE_CONTEXT_MARKER: &str = "@@";

/// One bounded transparency line, surfaced to the model as a leading line of
/// the tool output when the parser repaired a malformed-but-intent-clear
/// patch via the strict-first shape-repair pre-pass (apex-xt2.11).
const PATCH_REPAIR_NOTE: &str = "Note: the patch shape was repaired by the parser (missing or stray '*** Begin Patch'/'*** End Patch' boundary lines); the applied file content is exactly as provided.";

#[derive(Debug, PartialEq, Error, Clone)]
pub enum ParseError {
    #[error("invalid patch: {0}")]
    InvalidPatchError(String),
    #[error("invalid hunk at line {line_number}, {message}")]
    InvalidHunkError { message: String, line_number: usize },
}
use ParseError::*;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Hunk {
    AddFile {
        path: PathBuf,
        contents: String,
    },
    DeleteFile {
        path: PathBuf,
    },
    UpdateFile {
        path: PathBuf,
        move_path: Option<PathBuf>,

        /// Chunks should be in order, i.e. the `change_context` of one chunk
        /// should occur later in the file than the previous chunk.
        chunks: Vec<UpdateFileChunk>,
    },
}

impl Hunk {
    pub fn resolve_path(&self, cwd: &PathUri) -> Result<PathUri, PathUriParseError> {
        let path = match self {
            Hunk::UpdateFile { path, .. } => path,
            Hunk::AddFile { .. } | Hunk::DeleteFile { .. } => self.path(),
        };
        cwd.join(&path.to_string_lossy())
    }

    /// Returns the path affected by this hunk, using the move destination for rename hunks.
    pub fn path(&self) -> &Path {
        match self {
            Hunk::AddFile { path, .. } => path,
            Hunk::DeleteFile { path } => path,
            Hunk::UpdateFile {
                move_path: Some(path),
                ..
            } => path,
            Hunk::UpdateFile {
                path,
                move_path: None,
                ..
            } => path,
        }
    }
}

#[cfg(test)]
use Hunk::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct UpdateFileChunk {
    /// A single line of context used to narrow down the position of the chunk
    /// (this is usually a class, method, or function definition.)
    pub change_context: Option<String>,

    /// A contiguous block of lines that should be replaced with `new_lines`.
    /// `old_lines` must occur strictly after `change_context`.
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,

    /// Pairs of indices into `old_lines` and `new_lines` that identify lines
    /// parsed as context rather than inferred to be equal by their contents.
    pub context_line_indices: Vec<(usize, usize)>,

    /// If set to true, `old_lines` must occur at the end of the source file.
    /// (Tolerance around trailing newlines should be encouraged.)
    pub is_end_of_file: bool,
}

impl UpdateFileChunk {
    /// Adds a context line to both sides while recording its corresponding
    /// indices so it remains distinguishable from identical changed lines.
    pub(crate) fn push_context_line(&mut self, line: String) {
        self.context_line_indices
            .push((self.old_lines.len(), self.new_lines.len()));
        self.old_lines.push(line.clone());
        self.new_lines.push(line);
    }
}

pub fn parse_patch(patch: &str) -> Result<ApplyPatchArgs, ParseError> {
    match parse_patch_text(patch, ParseMode::Strict) {
        Ok(args) => Ok(args),
        Err(_strict_err) => {
            // Existing heredoc leniency (gpt-4.1 era) — unchanged behavior,
            // unchanged order. Its ERROR is the parity reference (R1-M1 [B]):
            // it is whatever today's Lenient path produces — boundary-class
            // or hunk-class for non-heredoc input, the INNER error for
            // heredoc-wrapped input (check_patch_boundaries_lenient re-calls
            // strict on the inner lines).
            let lenient_err = match parse_patch_text(patch, ParseMode::Lenient) {
                Ok(args) => return Ok(args),
                Err(e) => e,
            };
            // Fork ratchet (apex-xt2.11): strict-first shape repair, retry
            // ONCE. Gated on the lenient error; the fallback below returns
            // IT, so every currently-rejected input yields byte-identical
            // error text to today.
            if let Some(normalized) = normalize_patch_shape(&lenient_err, patch)
                && let Ok(mut args) = parse_patch_text(&normalized, ParseMode::Strict)
            {
                args.repair_note = Some(PATCH_REPAIR_NOTE.to_string());
                return Ok(args);
            }
            Err(lenient_err)
        }
    }
}

enum ParseMode {
    /// Parse the patch text argument as is.
    Strict,

    /// GPT-4.1 is known to formulate the `command` array for the `local_shell`
    /// tool call for `apply_patch` call using something like the following:
    ///
    /// ```json
    /// [
    ///   "apply_patch",
    ///   "<<'EOF'\n*** Begin Patch\n*** Update File: README.md\n@@...\n*** End Patch\nEOF\n",
    /// ]
    /// ```
    ///
    /// This is a problem because `local_shell` is a bit of a misnomer: the
    /// `command` is not invoked by passing the arguments to a shell like Bash,
    /// but are invoked using something akin to `execvpe(3)`.
    ///
    /// This is significant in this case because where a shell would interpret
    /// `<<'EOF'...` as a heredoc and pass the contents via stdin (which is
    /// fine, as `apply_patch` is specified to read from stdin if no argument is
    /// passed), `execvpe(3)` interprets the heredoc as a literal string. To get
    /// the `local_shell` tool to run a command the way shell would, the
    /// `command` array must be something like:
    ///
    /// ```json
    /// [
    ///   "bash",
    ///   "-lc",
    ///   "apply_patch <<'EOF'\n*** Begin Patch\n*** Update File: README.md\n@@...\n*** End Patch\nEOF\n",
    /// ]
    /// ```
    ///
    /// In lenient mode, we check if the argument to `apply_patch` starts with
    /// `<<'EOF'` and ends with `EOF\n`. If so, we strip off these markers,
    /// trim() the result, and treat what is left as the patch text.
    Lenient,
}

fn parse_patch_text(patch: &str, mode: ParseMode) -> Result<ApplyPatchArgs, ParseError> {
    let lines: Vec<&str> = patch.trim().lines().collect();
    let patch_lines = match mode {
        ParseMode::Strict => check_patch_boundaries_strict(&lines)?,
        ParseMode::Lenient => check_patch_boundaries_lenient(&lines)?,
    };

    let patch = patch_lines.join("\n");
    let mut parser = StreamingPatchParser::default();
    parser.push_delta(&patch)?;
    let hunks = parser.finish()?;
    let environment_id = parser.environment_id().map(str::to_owned);
    Ok(ApplyPatchArgs {
        hunks,
        patch,
        workdir: None,
        environment_id,
        repair_note: None,
    })
}

/// Strict-first shape-repair pre-pass (apex-xt2.11).
///
/// Given the lenient attempt's error and the raw patch text, deterministically
/// normalizes the patch for the two observed model shape signatures — shape A
/// (no `*** Begin Patch` anywhere, hunk header on line 1) and shape AB
/// (leading stray hunk header, then `*** Begin Patch` at index > 0) — and
/// returns the normalized text, or `None` when the input matches no
/// signature and the original error must stand unchanged.
fn normalize_patch_shape(err: &ParseError, patch: &str) -> Option<String> {
    // Boundary-class failures only: a hunk-class error means the patch body
    // already reached the state machine, and rewriting its lines would
    // silently change file contents.
    if !matches!(err, ParseError::InvalidPatchError(_)) {
        return None;
    }
    let lines: Vec<&str> = patch.trim().split('\n').collect();
    // Hunk-header-first signature (trimmed, like the boundary checks and the
    // state machine's own `trim`).
    let first_line = lines.first()?.trim();
    let hunk_header_first = first_line.starts_with(ADD_FILE_MARKER)
        || first_line.starts_with(DELETE_FILE_MARKER)
        || first_line.starts_with(UPDATE_FILE_MARKER);
    if !hunk_header_first {
        return None;
    }
    // Begin-line predicate: trimmed equality with the marker const — a
    // `+`-prefixed `+*** Begin Patch` content line is NOT a Begin line.
    let begin_index = lines
        .iter()
        .position(|line| line.trim() == BEGIN_PATCH_MARKER);
    match begin_index {
        // Case A (shape A): no `*** Begin Patch` anywhere — wrap the header
        // with a Begin line and, when absent, an End line.
        None => Some(with_end_boundary(
            std::iter::once(BEGIN_PATCH_MARKER).chain(lines.iter().copied()),
        )),
        // A Begin at index 0 is the canonical shape — the first strict
        // attempt already covered it; not leading-strip material.
        Some(0) => None,
        // Case AB (shape AB): a stray hunk header precedes the first Begin —
        // drop the leading stray lines, then the same End-append check.
        Some(i) => Some(with_end_boundary(lines[i..].iter().copied())),
    }
}

/// Joins the lines with `\n`, appending `*** End Patch` when the last line
/// is not it. Inputs reach here already trimmed, so the last line is
/// non-blank.
fn with_end_boundary<'a>(lines: impl Iterator<Item = &'a str>) -> String {
    let mut out: Vec<&str> = lines.collect();
    if out
        .last()
        .is_none_or(|line| line.trim() != END_PATCH_MARKER)
    {
        out.push(END_PATCH_MARKER);
    }
    out.join("\n")
}

/// Checks the start and end lines of the patch text for `apply_patch`,
/// returning an error if they do not match the expected markers.
fn check_patch_boundaries_strict<'a>(lines: &'a [&'a str]) -> Result<&'a [&'a str], ParseError> {
    let (first_line, last_line) = match lines {
        [] => (None, None),
        [first] => (Some(first), Some(first)),
        [first, .., last] => (Some(first), Some(last)),
    };
    check_start_and_end_lines_strict(first_line, last_line)?;
    Ok(lines)
}

/// If we are in lenient mode, we check if the first line starts with `<<EOF`
/// (possibly quoted) and the last line ends with `EOF`. There must be at least
/// 4 lines total because the heredoc markers take up 2 lines and the patch text
/// must have at least 2 lines.
///
/// If successful, returns the lines of the patch text that contain the patch
/// contents, excluding the heredoc markers.
fn check_patch_boundaries_lenient<'a>(
    original_lines: &'a [&'a str],
) -> Result<&'a [&'a str], ParseError> {
    let original_parse_error = match check_patch_boundaries_strict(original_lines) {
        Ok(lines) => return Ok(lines),
        Err(e) => e,
    };

    match original_lines {
        [first, .., last] => {
            if (first == &"<<EOF" || first == &"<<'EOF'" || first == &"<<\"EOF\"")
                && last.ends_with("EOF")
                && original_lines.len() >= 4
            {
                let inner_lines = &original_lines[1..original_lines.len() - 1];
                check_patch_boundaries_strict(inner_lines)
            } else {
                Err(original_parse_error)
            }
        }
        _ => Err(original_parse_error),
    }
}

fn check_start_and_end_lines_strict(
    first_line: Option<&&str>,
    last_line: Option<&&str>,
) -> Result<(), ParseError> {
    let first_line = first_line.map(|line| line.trim());
    let last_line = last_line.map(|line| line.trim());

    match (first_line, last_line) {
        (Some(first), Some(last)) if first == BEGIN_PATCH_MARKER && last == END_PATCH_MARKER => {
            Ok(())
        }
        (Some(first), _) if first != BEGIN_PATCH_MARKER => Err(InvalidPatchError(String::from(
            "The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'.",
        ))),
        _ => Err(InvalidPatchError(String::from(
            "The last line of the patch must be '*** End Patch'. The terminator line is exactly '*** End Patch' with no '+' or other prefix and no lines after it.",
        ))),
    }
}

#[test]
fn test_parse_patch() {
    assert_eq!(
        parse_patch_text("bad", ParseMode::Strict),
        Err(InvalidPatchError(
            "The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'.".to_string()
        ))
    );
    assert_eq!(
        parse_patch_text("*** Begin Patch\nbad", ParseMode::Strict),
        Err(InvalidPatchError(
            "The last line of the patch must be '*** End Patch'. The terminator line is exactly '*** End Patch' with no '+' or other prefix and no lines after it.".to_string()
        ))
    );

    assert_eq!(
        parse_patch_text(
            concat!(
                "*** Begin Patch",
                " ",
                "\n*** Add File: foo\n+hi\n",
                " ",
                "*** End Patch"
            ),
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![AddFile {
            path: PathBuf::from("foo"),
            contents: "hi\n".to_string()
        }]
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Update File: test.py\n\
             *** End Patch",
            ParseMode::Strict
        ),
        Err(InvalidHunkError {
            message: "Update file hunk for path 'test.py' is empty".to_string(),
            line_number: 2,
        })
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        Vec::new()
    );
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Add File: path/add.py\n\
             +abc\n\
             +def\n\
             *** Delete File: path/delete.py\n\
             *** Update File: path/update.py\n\
             *** Move to: path/update2.py\n\
             @@ def f():\n\
             -    pass\n\
             +    return 123\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![
            AddFile {
                path: PathBuf::from("path/add.py"),
                contents: "abc\ndef\n".to_string()
            },
            DeleteFile {
                path: PathBuf::from("path/delete.py")
            },
            UpdateFile {
                path: PathBuf::from("path/update.py"),
                move_path: Some(PathBuf::from("path/update2.py")),
                chunks: vec![UpdateFileChunk {
                    change_context: Some("def f():".to_string()),
                    old_lines: vec!["    pass".to_string()],
                    new_lines: vec!["    return 123".to_string()],
                    context_line_indices: vec![],
                    is_end_of_file: false
                }]
            }
        ]
    );
    // Update hunk followed by another hunk (Add File).
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Update File: file.py\n\
             @@\n\
             +line\n\
             *** Add File: other.py\n\
             +content\n\
             *** End Patch",
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![
            UpdateFile {
                path: PathBuf::from("file.py"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: vec![],
                    new_lines: vec!["line".to_string()],
                    context_line_indices: vec![],
                    is_end_of_file: false
                }],
            },
            AddFile {
                path: PathBuf::from("other.py"),
                contents: "content\n".to_string()
            }
        ]
    );

    // Update hunk without an explicit @@ header for the first chunk should parse.
    // Use a raw string to preserve the leading space diff marker on the context line.
    assert_eq!(
        parse_patch_text(
            r#"*** Begin Patch
*** Update File: file2.py
 import foo
+bar
*** End Patch"#,
            ParseMode::Strict
        )
        .unwrap()
        .hunks,
        vec![UpdateFile {
            path: PathBuf::from("file2.py"),
            move_path: None,
            chunks: vec![UpdateFileChunk {
                change_context: None,
                old_lines: vec!["import foo".to_string()],
                new_lines: vec!["import foo".to_string(), "bar".to_string()],
                context_line_indices: vec![(0, 0)],
                is_end_of_file: false,
            }],
        }]
    );
}

#[test]
fn test_parse_patch_preserves_end_of_file_marker() {
    let patch =
        "*** Begin Patch\n*** Update File: file.txt\n@@\n+quux\n*** End of File\n\n*** End Patch";
    assert_eq!(
        parse_patch(patch),
        Ok(ApplyPatchArgs {
            hunks: vec![UpdateFile {
                path: PathBuf::from("file.txt"),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: Vec::new(),
                    new_lines: vec!["quux".to_string()],
                    context_line_indices: vec![],
                    is_end_of_file: true,
                }],
            }],
            patch: patch.to_string(),
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );
}

#[test]
fn test_parse_patch_accepts_relative_and_absolute_hunk_paths() {
    let dir = tempfile::tempdir().unwrap();
    let absolute_delete = dir.path().join("absolute-delete.py").abs();
    let absolute_update = dir.path().join("absolute-update.py").abs();
    let patch_text = format!(
        r#"*** Begin Patch
*** Add File: relative-add.py
+content
*** Delete File: {}
*** Update File: {}
@@
-old
+new
*** End Patch"#,
        absolute_delete.display(),
        absolute_update.display()
    );

    assert_eq!(
        parse_patch_text(&patch_text, ParseMode::Strict)
            .unwrap()
            .hunks,
        vec![
            AddFile {
                path: PathBuf::from("relative-add.py"),
                contents: "content\n".to_string()
            },
            DeleteFile {
                path: absolute_delete.to_path_buf()
            },
            UpdateFile {
                path: absolute_update.to_path_buf(),
                move_path: None,
                chunks: vec![UpdateFileChunk {
                    change_context: None,
                    old_lines: vec!["old".to_string()],
                    new_lines: vec!["new".to_string()],
                    context_line_indices: vec![],
                    is_end_of_file: false
                }]
            },
        ]
    );
}

#[test]
fn test_hunk_resolve_path_accepts_relative_and_absolute_paths() {
    let cwd_dir = tempfile::tempdir().unwrap();
    let cwd = PathUri::from_host_native_path(cwd_dir.path()).unwrap();
    let absolute_dir = tempfile::tempdir().unwrap();
    let absolute_add = absolute_dir.path().join("absolute-add.py").abs();
    let absolute_delete = absolute_dir.path().join("absolute-delete.py").abs();
    let absolute_update = absolute_dir.path().join("absolute-update.py").abs();

    for (hunk, expected_path) in [
        (
            AddFile {
                path: PathBuf::from("relative-add.py"),
                contents: String::new(),
            },
            cwd.join("relative-add.py").unwrap(),
        ),
        (
            DeleteFile {
                path: PathBuf::from("relative-delete.py"),
            },
            cwd.join("relative-delete.py").unwrap(),
        ),
        (
            UpdateFile {
                path: PathBuf::from("relative-update.py"),
                move_path: None,
                chunks: Vec::new(),
            },
            cwd.join("relative-update.py").unwrap(),
        ),
        (
            AddFile {
                path: absolute_add.to_path_buf(),
                contents: String::new(),
            },
            PathUri::from_abs_path(&absolute_add),
        ),
        (
            DeleteFile {
                path: absolute_delete.to_path_buf(),
            },
            PathUri::from_abs_path(&absolute_delete),
        ),
        (
            UpdateFile {
                path: absolute_update.to_path_buf(),
                move_path: None,
                chunks: Vec::new(),
            },
            PathUri::from_abs_path(&absolute_update),
        ),
    ] {
        assert_eq!(hunk.resolve_path(&cwd), Ok(expected_path));
    }
}

#[test]
fn test_parse_patch_lenient() {
    let patch_text = r#"*** Begin Patch
*** Update File: file2.py
 import foo
+bar
*** End Patch"#;
    let expected_patch = vec![UpdateFile {
        path: PathBuf::from("file2.py"),
        move_path: None,
        chunks: vec![UpdateFileChunk {
            change_context: None,
            old_lines: vec!["import foo".to_string()],
            new_lines: vec!["import foo".to_string(), "bar".to_string()],
            context_line_indices: vec![(0, 0)],
            is_end_of_file: false,
        }],
    }];
    let expected_error =
        InvalidPatchError(
            "The first line of the patch must be '*** Begin Patch'. The patch body starts on the next line with a hunk header (e.g. '*** Add File: <path>') and ends with the line '*** End Patch'.".to_string(),
        );

    let patch_text_in_heredoc = format!("<<EOF\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_heredoc, ParseMode::Lenient),
        Ok(ApplyPatchArgs {
            hunks: expected_patch.clone(),
            patch: patch_text.to_string(),
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );

    let patch_text_in_single_quoted_heredoc = format!("<<'EOF'\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_single_quoted_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_single_quoted_heredoc, ParseMode::Lenient),
        Ok(ApplyPatchArgs {
            hunks: expected_patch.clone(),
            patch: patch_text.to_string(),
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );

    let patch_text_in_double_quoted_heredoc = format!("<<\"EOF\"\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_double_quoted_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_double_quoted_heredoc, ParseMode::Lenient),
        Ok(ApplyPatchArgs {
            hunks: expected_patch,
            patch: patch_text.to_string(),
            workdir: None,
            environment_id: None,
            repair_note: None,
        })
    );

    let patch_text_in_mismatched_quotes_heredoc = format!("<<\"EOF'\n{patch_text}\nEOF\n");
    assert_eq!(
        parse_patch_text(&patch_text_in_mismatched_quotes_heredoc, ParseMode::Strict),
        Err(expected_error.clone())
    );
    assert_eq!(
        parse_patch_text(&patch_text_in_mismatched_quotes_heredoc, ParseMode::Lenient),
        Err(expected_error.clone())
    );

    let patch_text_with_missing_closing_heredoc =
        "<<EOF\n*** Begin Patch\n*** Update File: file2.py\nEOF\n".to_string();
    assert_eq!(
        parse_patch_text(&patch_text_with_missing_closing_heredoc, ParseMode::Strict),
        Err(expected_error)
    );
    assert_eq!(
        parse_patch_text(&patch_text_with_missing_closing_heredoc, ParseMode::Lenient),
        Err(InvalidPatchError(
            "The last line of the patch must be '*** End Patch'. The terminator line is exactly '*** End Patch' with no '+' or other prefix and no lines after it.".to_string()
        ))
    );
}

#[test]
fn test_parse_patch_environment_id_preamble() {
    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Environment ID: remote\n\
             *** Add File: hello.txt\n\
             +hello\n\
             *** End Patch",
            ParseMode::Strict
        ),
        Ok(ApplyPatchArgs {
            hunks: vec![AddFile {
                path: PathBuf::from("hello.txt"),
                contents: "hello\n".to_string(),
            }],
            patch: "*** Begin Patch\n*** Environment ID: remote\n*** Add File: hello.txt\n+hello\n*** End Patch".to_string(),
            workdir: None,
            environment_id: Some("remote".to_string()),
            repair_note: None,
        })
    );

    assert_eq!(
        parse_patch_text(
            "*** Begin Patch\n\
             *** Environment ID:   \n\
             *** Add File: hello.txt\n\
             +hello\n\
             *** End Patch",
            ParseMode::Strict
        ),
        Err(InvalidPatchError(
            "apply_patch environment_id cannot be empty".to_string()
        ))
    );
}

#[cfg(test)]
#[path = "parser_shape_retry_tests.rs"]
mod parser_shape_retry_tests;
