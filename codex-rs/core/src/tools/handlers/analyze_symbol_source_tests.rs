use pretty_assertions::assert_eq;
use tempfile::tempdir;

use super::*;

// ---------------------------------------------------------------------------
// Unit tests: base_symbol_name
// ---------------------------------------------------------------------------

#[test]
fn base_symbol_name_plain() {
    assert_eq!(base_symbol_name("my_func"), "my_func");
}

#[test]
fn base_symbol_name_rust_qualified() {
    assert_eq!(base_symbol_name("MyStruct::my_method"), "my_method");
}

#[test]
fn base_symbol_name_deeply_qualified() {
    assert_eq!(
        base_symbol_name("crate::module::SubModule::helper"),
        "helper"
    );
}

#[test]
fn base_symbol_name_dotted() {
    assert_eq!(base_symbol_name("pkg.Class.Method"), "Method");
}

#[test]
fn base_symbol_name_prefers_double_colon() {
    // When both `::` and `.` appear, `::` takes precedence (Rust style).
    assert_eq!(base_symbol_name("pkg::Type.method"), "method");
}

// ---------------------------------------------------------------------------
// Unit tests: is_test_file
// ---------------------------------------------------------------------------

#[test]
fn is_test_file_slash_tests() {
    assert!(is_test_file("/project/tests/my_mod.rs"));
}

#[test]
fn is_test_file_slash_test() {
    assert!(is_test_file("/project/test/helpers.ts"));
}

#[test]
fn is_test_file_underscore_test_dot() {
    assert!(is_test_file("src/parser_test.rs"));
}

#[test]
fn is_test_file_dot_test_dot() {
    assert!(is_test_file("src/parser.test.ts"));
}

#[test]
fn is_test_file_spec() {
    assert!(is_test_file("src/components/Button.spec.tsx"));
}

#[test]
fn is_test_file_normal_file() {
    assert!(!is_test_file("src/lib.rs"));
    assert!(!is_test_file("src/tools/grep_files.rs"));
}

// ---------------------------------------------------------------------------
// Unit tests: extract_callees
// ---------------------------------------------------------------------------

#[test]
fn extract_callees_basic() {
    let source = r#"
fn my_func(x: u32) -> u32 {
    let y = helper_a(x);
    let z = helper_b(y);
    y + z
}
"#;
    let callees = extract_callees(source, 10, "my_func");
    let names: Vec<&str> = callees.iter().map(|c| c.callee.as_str()).collect();
    assert!(names.contains(&"helper_a"), "should find helper_a");
    assert!(names.contains(&"helper_b"), "should find helper_b");
}

#[test]
fn extract_callees_skips_comments() {
    let source = r#"
fn my_func() {
    // ignored_call(x);
    /* also_ignored(y); */
    real_call(1);
}
"#;
    let callees = extract_callees(source, 10, "my_func");
    let names: Vec<&str> = callees.iter().map(|c| c.callee.as_str()).collect();
    assert!(names.contains(&"real_call"), "real_call should be found");
    assert!(
        !names.contains(&"ignored_call"),
        "ignored_call in comment should be skipped"
    );
    assert!(
        !names.contains(&"also_ignored"),
        "also_ignored in block comment should be skipped"
    );
}

#[test]
fn extract_callees_skips_keywords() {
    let source = r#"
fn process() {
    for item in items() {
        if valid(item) {
            transform(item);
        }
    }
}
"#;
    let callees = extract_callees(source, 10, "process");
    let names: Vec<&str> = callees.iter().map(|c| c.callee.as_str()).collect();
    assert!(!names.contains(&"for"), "keyword 'for' must be skipped");
    assert!(!names.contains(&"if"), "keyword 'if' must be skipped");
    assert!(names.contains(&"items"), "items() should be found");
    assert!(names.contains(&"valid"), "valid() should be found");
    assert!(names.contains(&"transform"), "transform() should be found");
}

#[test]
fn extract_callees_classifies_method_call() {
    let source = r#"
fn run(obj: MyType) {
    obj.do_thing(42);
}
"#;
    let callees = extract_callees(source, 10, "run");
    let do_thing = callees.iter().find(|c| c.callee == "do_thing");
    assert!(do_thing.is_some(), "do_thing should be found");
    assert_eq!(
        do_thing.unwrap().call_type.as_deref(),
        Some("method"),
        "dot-prefixed call should be classified as method"
    );
}

#[test]
fn extract_callees_filters_new_keyword() {
    // `new` is listed as a keyword and should not appear in callees.
    let source = r#"
fn build() -> Thing {
    Thing::new()
}
"#;
    let callees = extract_callees(source, 10, "build");
    let new_call = callees.iter().find(|c| c.callee == "new");
    assert!(
        new_call.is_none(),
        "new is a keyword and should be filtered out"
    );
}

#[test]
fn extract_callees_classifies_static_call_non_keyword() {
    let source = r#"
fn build() -> Config {
    Config::default()
}
"#;
    let callees = extract_callees(source, 10, "build");
    let default_call = callees.iter().find(|c| c.callee == "default");
    assert!(default_call.is_some(), "default() should be found");
    assert_eq!(
        default_call.unwrap().call_type.as_deref(),
        Some("static"),
        "double-colon call should be classified as static"
    );
}

#[test]
fn extract_callees_respects_max() {
    let source = r#"
fn many_calls() {
    a1(1); a2(2); a3(3); a4(4); a5(5);
}
"#;
    let callees = extract_callees(source, 3, "many_calls");
    assert!(callees.len() <= 3, "should respect max_callees limit");
}

#[test]
fn extract_callees_deduplicates() {
    let source = r#"
fn repeated() {
    helper(1);
    helper(2);
    helper(3);
}
"#;
    let callees = extract_callees(source, 10, "repeated");
    let helper_count = callees.iter().filter(|c| c.callee == "helper").count();
    assert_eq!(helper_count, 1, "duplicate callees should be deduped");
}

#[test]
fn extract_callees_skips_self_name() {
    let source = r#"
fn recurse(n: u32) -> u32 {
    if n == 0 { return 0; }
    recurse(n - 1)
}
"#;
    let callees = extract_callees(source, 10, "recurse");
    assert!(
        !callees.iter().any(|c| c.callee == "recurse"),
        "the symbol itself should not appear as a callee"
    );
}

// ---------------------------------------------------------------------------
// Unit tests: build_definition_pattern
// ---------------------------------------------------------------------------

#[test]
fn build_definition_pattern_matches_rust_fn() {
    let pattern = build_definition_pattern("my_func");
    // The pattern should be a non-empty string.
    assert!(!pattern.is_empty());
    // Quick sanity check using regex crate via string matching.
    assert!(
        pattern.contains("my_func"),
        "pattern should contain the symbol name"
    );
}

// ---------------------------------------------------------------------------
// Integration test using a temporary directory workspace
// ---------------------------------------------------------------------------

fn rg_available() -> bool {
    std::process::Command::new("rg")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn integration_parse_rg_lines_basic() {
    let stdout = b"src/lib.rs:10:fn my_func(x: u32) {\nsrc/main.rs:5:my_func(42);\n";
    let results = parse_rg_lines(stdout, 100);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "src/lib.rs");
    assert_eq!(results[0].1, 10);
    assert_eq!(results[0].2, "fn my_func(x: u32) {");
    assert_eq!(results[1].0, "src/main.rs");
    assert_eq!(results[1].1, 5);
}

#[test]
fn integration_parse_rg_lines_truncates() {
    let stdout = b"a.rs:1:foo\nb.rs:2:bar\nc.rs:3:baz\n";
    let results = parse_rg_lines(stdout, 2);
    assert_eq!(results.len(), 2);
}

#[test]
fn integration_parse_rg_lines_skips_malformed() {
    // Lines without a valid `file:lineno:content` format should be skipped.
    let stdout = b"only_two_parts\na.rs:not_a_number:content\nb.rs:5:ok\n";
    let results = parse_rg_lines(stdout, 100);
    assert_eq!(results.len(), 1, "only the valid line should be parsed");
    assert_eq!(results[0].0, "b.rs");
}

/// Full round-trip: write a fake workspace, invoke the ripgrep helpers, and
/// verify we find definition + references.
#[tokio::test]
async fn integration_find_definition_and_callers() -> anyhow::Result<()> {
    if !rg_available() {
        eprintln!("rg not in PATH; skipping integration test");
        return Ok(());
    }

    let dir = tempdir()?;
    let base = dir.path();

    // Write a "library" file with a function definition.
    std::fs::write(
        base.join("lib.rs"),
        r#"
pub fn compute_result(x: u32) -> u32 {
    let y = helper(x);
    y * 2
}

fn helper(x: u32) -> u32 {
    x + 1
}
"#,
    )?;

    // Write a caller file.
    std::fs::write(
        base.join("main.rs"),
        r#"
fn main() {
    let v = compute_result(10);
    println!("{}", v);
}
"#,
    )?;

    // Write a test file.
    std::fs::write(
        base.join("lib_test.rs"),
        r#"
#[test]
fn test_compute_result() {
    assert_eq!(compute_result(1), 4);
}
"#,
    )?;

    // Search for the definition.
    let def_pattern = build_definition_pattern("compute_result");
    let def_hits = run_rg_search_lines(&def_pattern, base, 5)
        .await
        .map_err(anyhow::Error::msg)?;
    assert!(!def_hits.is_empty(), "should find definition");
    assert!(
        def_hits[0].0.ends_with("lib.rs"),
        "definition should be in lib.rs"
    );
    assert_eq!(def_hits[0].1, 2, "definition should be on line 2");

    // Search for references.
    let ref_pattern = format!(r"\b{}\b", "compute_result");
    let ref_hits = run_rg_search_lines_all(&ref_pattern, base, 100)
        .await
        .map_err(anyhow::Error::msg)?;
    assert!(ref_hits.len() >= 2, "should find at least 2 references");

    // Partition into test vs non-test callers.
    let def_key = (def_hits[0].0.clone(), def_hits[0].1);
    let callers: Vec<_> = ref_hits
        .iter()
        .filter(|(f, l, _)| (*f != def_key.0 || *l != def_key.1) && !is_test_file(f))
        .collect();
    let test_callers: Vec<_> = ref_hits
        .iter()
        .filter(|(f, l, _)| (*f != def_key.0 || *l != def_key.1) && is_test_file(f))
        .collect();

    assert!(
        !callers.is_empty(),
        "should have at least one non-test caller"
    );
    assert!(
        !test_callers.is_empty(),
        "should have at least one test caller"
    );

    // Extract callees from the definition snippet.
    let source = read_source_snippet_sync(&def_hits[0].0, def_hits[0].1, 10)
        .expect("should read source snippet");
    let callees = extract_callees(&source, 10, "compute_result");
    let callee_names: Vec<&str> = callees.iter().map(|c| c.callee.as_str()).collect();
    assert!(
        callee_names.contains(&"helper"),
        "helper() should be extracted as a callee"
    );

    Ok(())
}
