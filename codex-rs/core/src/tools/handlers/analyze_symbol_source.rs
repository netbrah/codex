use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;
use tokio::time::timeout;

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::ToolHandler;
use crate::tools::registry::ToolKind;

pub struct AnalyzeSymbolSourceHandler;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum number of files in the scope before refusing to search.
const MAX_FILE_SCOPE: usize = 50_000;

fn default_max_callers() -> usize {
    15
}

fn default_max_callees() -> usize {
    20
}

fn default_context_lines() -> usize {
    50
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnalyzeSymbolSourceArgs {
    symbol: String,
    #[serde(default)]
    scope_path: Option<String>,
    #[serde(default = "default_max_callers")]
    max_callers: usize,
    #[serde(default = "default_max_callees")]
    max_callees: usize,
    #[serde(default = "default_context_lines")]
    context_lines: usize,
    #[serde(default = "default_true")]
    include_source: bool,
    #[serde(default = "default_true")]
    include_tests: bool,
    #[serde(default)]
    verbose: bool,
}

#[derive(Serialize)]
pub struct DefinitionResult {
    pub file: String,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct CallerResult {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CalleeResult {
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(rename = "callType", skip_serializing_if = "Option::is_none")]
    pub call_type: Option<String>,
}

#[derive(Serialize)]
struct AnalysisOutput {
    success: bool,
    symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    definition: Option<DefinitionResult>,
    callers: Vec<CallerResult>,
    #[serde(rename = "testCallers", skip_serializing_if = "Option::is_none")]
    test_callers: Option<Vec<CallerResult>>,
    callees: Vec<CalleeResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timing: Option<TimingInfo>,
}

#[derive(Serialize)]
struct TimingInfo {
    definition_ms: u64,
    references_ms: u64,
    total_ms: u64,
}

#[async_trait]
impl ToolHandler for AnalyzeSymbolSourceHandler {
    type Output = FunctionToolOutput;

    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<Self::Output, FunctionCallError> {
        let ToolInvocation { payload, turn, .. } = invocation;

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "analyze_symbol_source handler received unsupported payload".to_string(),
                ));
            }
        };

        let args: AnalyzeSymbolSourceArgs = parse_arguments(&arguments)?;

        let symbol = args.symbol.trim().to_string();
        if symbol.is_empty() {
            return Err(FunctionCallError::RespondToModel(
                "symbol must not be empty".to_string(),
            ));
        }

        let scope_path = turn.resolve_path(args.scope_path.clone());

        // File-count governor: avoid runaway scans on huge workspaces.
        if let Ok(count) = count_files_in_scope(&scope_path).await
            && count > MAX_FILE_SCOPE {
                let output = AnalysisOutput {
                    success: false,
                    symbol: symbol.clone(),
                    definition: None,
                    callers: vec![],
                    test_callers: None,
                    callees: vec![],
                    error: Some(format!(
                        "Scope too broad: {count} files found under `{}`. \
                         Provide a narrower `scopePath` (e.g. a sub-directory) to limit the search.",
                        scope_path.display()
                    )),
                    timing: None,
                };
                let json = serde_json::to_string_pretty(&output)
                    .unwrap_or_else(|_| "{\"success\":false}".to_string());
                return Ok(FunctionToolOutput::from_text(json, Some(false)));
            }

        let total_start = Instant::now();

        // Phase 1: definition search.
        let def_start = Instant::now();
        let base_name = base_symbol_name(&symbol).to_string();
        let def_pattern = build_definition_pattern(&base_name);
        let definition_ms;
        let definition = match run_rg_search_lines(&def_pattern, &scope_path, 20).await {
            Ok(hits) => {
                definition_ms = def_start.elapsed().as_millis() as u64;
                hits.into_iter().next().map(|(file, line, content)| {
                    let signature = content.trim().to_string();
                    let source = if args.include_source {
                        read_source_snippet_sync(&file, line, args.context_lines)
                    } else {
                        None
                    };
                    DefinitionResult {
                        file,
                        line,
                        signature: if signature.is_empty() {
                            None
                        } else {
                            Some(signature)
                        },
                        source,
                    }
                })
            }
            Err(_) => {
                definition_ms = def_start.elapsed().as_millis() as u64;
                None
            }
        };

        // Phase 2: reference search for callers.
        let ref_start = Instant::now();
        let mut callers: Vec<CallerResult> = vec![];
        let mut test_callers: Vec<CallerResult> = vec![];

        if args.max_callers > 0 {
            let ref_pattern = format!(r"\b{}\b", regex_escape(&base_name));
            if let Ok(hits) =
                run_rg_search_lines_all(&ref_pattern, &scope_path, args.max_callers * 10).await
            {
                let def_key = definition.as_ref().map(|d| (d.file.clone(), d.line));
                let mut seen: HashSet<(String, u32)> = HashSet::new();
                for (file, line, content) in hits {
                    let key = (file.clone(), line);
                    if Some(&key) == def_key.as_ref() {
                        continue;
                    }
                    if !seen.insert(key) {
                        continue;
                    }
                    let caller = CallerResult {
                        file: file.clone(),
                        line: Some(line),
                        context: Some(content.trim().to_string()),
                    };
                    if is_test_file(&file) {
                        test_callers.push(caller);
                    } else {
                        callers.push(caller);
                    }
                }
                callers.truncate(args.max_callers);
                test_callers.truncate(args.max_callers);
            }
        }
        let references_ms = ref_start.elapsed().as_millis() as u64;

        // Phase 3: callee extraction from the definition snippet.
        let callees = definition
            .as_ref()
            .and_then(|d| d.source.as_deref())
            .map(|src| extract_callees(src, args.max_callees, &base_name))
            .unwrap_or_default();

        let total_ms = total_start.elapsed().as_millis() as u64;

        let timing = if args.verbose {
            Some(TimingInfo {
                definition_ms,
                references_ms,
                total_ms,
            })
        } else {
            None
        };

        let test_callers_out = if args.include_tests && !test_callers.is_empty() {
            Some(test_callers)
        } else {
            None
        };

        let output = AnalysisOutput {
            success: true,
            symbol,
            definition,
            callers,
            test_callers: test_callers_out,
            callees,
            error: None,
            timing,
        };

        let json = serde_json::to_string_pretty(&output).map_err(|e| {
            FunctionCallError::RespondToModel(format!("failed to serialize output: {e}"))
        })?;

        Ok(FunctionToolOutput::from_text(json, Some(true)))
    }
}

// ---------------------------------------------------------------------------
// Public helpers (also exercised by tests)
// ---------------------------------------------------------------------------

/// Returns the base name of a (possibly qualified) symbol.
///
/// For `MyStruct::my_method` returns `my_method`.
/// For `pkg.MyStruct.Method` returns `Method`.
/// For plain `my_func` returns `my_func`.
/// When both `::` and `.` appear, the rightmost separator wins.
pub fn base_symbol_name(symbol: &str) -> &str {
    let colon_end = symbol.rfind("::").map(|i| i + 2);
    let dot_end = symbol.rfind('.').map(|i| i + 1);
    match (colon_end, dot_end) {
        (Some(c), Some(d)) => &symbol[c.max(d)..],
        (Some(c), None) => &symbol[c..],
        (None, Some(d)) => &symbol[d..],
        (None, None) => symbol,
    }
}

/// Returns `true` if the file path looks like a test file.
pub fn is_test_file(path: &str) -> bool {
    let p = path.to_lowercase();
    p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("_test.")
        || p.contains(".test.")
        || p.contains(".spec.")
        || p.ends_with("_test")
        || p.ends_with("_tests")
}

/// Build a ripgrep-compatible regex pattern that matches likely definition lines
/// for the given base symbol name.
pub fn build_definition_pattern(base_name: &str) -> String {
    let escaped = regex_escape(base_name);
    // Covers Rust (fn/struct/trait/enum/type/const/impl), Python (def/class),
    // JS/TS (function/class/const/let), Go (func), and more.
    format!(
        r"^\s*(pub(\s*\([^)]*\))?\s+|async\s+|pub\s+async\s+|export\s+)?(fn|def|class|struct|trait|interface|function|type|enum|const|let|var|func)\s+{escaped}\s*[<({{]"
    )
}

/// Extract callee (function/method call) names from a source snippet.
///
/// Skips comment lines and language keywords to reduce noise.
pub fn extract_callees(source: &str, max_callees: usize, self_name: &str) -> Vec<CalleeResult> {
    const KEYWORDS: &[&str] = &[
        "if",
        "while",
        "for",
        "loop",
        "match",
        "switch",
        "catch",
        "try",
        "return",
        "assert",
        "panic",
        "todo",
        "unimplemented",
        "unreachable",
        "println",
        "eprintln",
        "print",
        "eprint",
        "format",
        "write",
        "writeln",
        "vec",
        "box",
        "drop",
        "clone",
        "new",
    ];

    let mut results: Vec<CalleeResult> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for (idx, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();

        // Skip blank or comment-only lines.
        if line.is_empty()
            || line.starts_with("//")
            || line.starts_with('#')
            || line.starts_with("/*")
            || line.starts_with('*')
            || line.starts_with("--")
        {
            continue;
        }

        let code = strip_inline_comment(line);

        // Walk the line looking for `identifier(` patterns.
        let mut pos = 0;
        while pos < code.len() {
            let Some(rel_paren) = code[pos..].find('(') else {
                break;
            };
            let abs_paren = pos + rel_paren;
            let before = &code[..abs_paren];
            let ident = extract_identifier_before(before);
            pos = abs_paren + 1;

            if ident.is_empty() || ident.len() <= 1 {
                continue;
            }

            let call_type = if before.ends_with(&format!(".{ident}")) {
                "method"
            } else if before.ends_with(&format!("::{ident}")) {
                "static"
            } else {
                "function"
            };

            let lower = ident.to_lowercase();
            if KEYWORDS.contains(&lower.as_str()) || ident == self_name {
                continue;
            }

            if seen.insert(ident.clone()) {
                results.push(CalleeResult {
                    callee: ident,
                    line: Some((idx + 1) as u32),
                    call_type: Some(call_type.to_string()),
                });
                if results.len() >= max_callees {
                    return results;
                }
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Escape regex special characters so an identifier can be used as a literal.
fn regex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if matches!(
            ch,
            '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// Strip an inline `//` or ` #` comment from a code line (best-effort).
fn strip_inline_comment(line: &str) -> &str {
    if let Some(idx) = line.find("//") {
        return &line[..idx];
    }
    if let Some(idx) = line.find(" #") {
        return &line[..idx];
    }
    line
}

/// Given the text before an opening `(`, extract the trailing identifier.
fn extract_identifier_before(before: &str) -> String {
    let trimmed = before.trim_end();
    let bytes = trimmed.as_bytes();
    let mut end = bytes.len();
    while end > 0 && is_ident_char(bytes[end - 1]) {
        end -= 1;
    }
    trimmed[end..].to_string()
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Run ripgrep with `--max-count 1` per file (good for definition search).
async fn run_rg_search_lines(
    pattern: &str,
    search_path: &Path,
    max_results: usize,
) -> Result<Vec<(String, u32, String)>, String> {
    let output = timeout(
        COMMAND_TIMEOUT,
        Command::new("rg")
            .arg("--no-heading")
            .arg("-n")
            .arg("--max-count")
            .arg("1")
            .arg("--regexp")
            .arg(pattern)
            .arg("--no-messages")
            .arg("--")
            .arg(search_path)
            .output(),
    )
    .await
    .map_err(|_| "rg timed out after 30 seconds".to_string())?
    .map_err(|e| format!("failed to launch rg: {e}"))?;

    match output.status.code() {
        Some(0) => Ok(parse_rg_lines(&output.stdout, max_results)),
        Some(1) => Ok(vec![]),
        _ => Err(format!(
            "rg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
    }
}

/// Run ripgrep without per-file result cap (good for reference/caller search).
async fn run_rg_search_lines_all(
    pattern: &str,
    search_path: &Path,
    max_results: usize,
) -> Result<Vec<(String, u32, String)>, String> {
    let output = timeout(
        COMMAND_TIMEOUT,
        Command::new("rg")
            .arg("--no-heading")
            .arg("-n")
            .arg("--regexp")
            .arg(pattern)
            .arg("--no-messages")
            .arg("--")
            .arg(search_path)
            .output(),
    )
    .await
    .map_err(|_| "rg timed out after 30 seconds".to_string())?
    .map_err(|e| format!("failed to launch rg: {e}"))?;

    match output.status.code() {
        Some(0) => Ok(parse_rg_lines(&output.stdout, max_results)),
        Some(1) => Ok(vec![]),
        _ => Err(format!(
            "rg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
    }
}

/// Parse `filename:lineno:content` lines from ripgrep output.
fn parse_rg_lines(stdout: &[u8], max_results: usize) -> Vec<(String, u32, String)> {
    let mut results = Vec::new();
    for raw in stdout.split(|&b| b == b'\n') {
        if raw.is_empty() {
            continue;
        }
        let Ok(line) = std::str::from_utf8(raw) else {
            continue;
        };
        // Format: "path/to/file.rs:42:content here"
        let mut parts = line.splitn(3, ':');
        let (Some(file), Some(lineno_str), Some(content)) =
            (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let Ok(lineno) = lineno_str.parse::<u32>() else {
            continue;
        };
        results.push((file.to_string(), lineno, content.to_string()));
        if results.len() >= max_results {
            break;
        }
    }
    results
}

/// Count files under a scope path using `rg --files`.
async fn count_files_in_scope(scope_path: &Path) -> Result<usize, String> {
    let output = timeout(
        COMMAND_TIMEOUT,
        Command::new("rg")
            .arg("--files")
            .arg("--no-messages")
            .arg("--")
            .arg(scope_path)
            .output(),
    )
    .await
    .map_err(|_| "rg --files timed out".to_string())?
    .map_err(|e| format!("failed to launch rg: {e}"))?;

    match output.status.code() {
        Some(0) | Some(1) => Ok(output.stdout.iter().filter(|&&b| b == b'\n').count()),
        _ => Err(format!(
            "rg --files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
    }
}

/// Read `context_lines` lines of source around `line_number` from a file.
fn read_source_snippet_sync(file: &str, line_number: u32, context_lines: usize) -> Option<String> {
    let content = std::fs::read_to_string(file).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let zero_idx = (line_number as usize).saturating_sub(1);
    // Start a couple lines before to capture doc comments / attributes.
    let start = zero_idx.saturating_sub(2);
    let end = (zero_idx + context_lines).min(lines.len());
    Some(lines[start..end].join("\n"))
}

#[cfg(test)]
#[path = "analyze_symbol_source_tests.rs"]
mod tests;
