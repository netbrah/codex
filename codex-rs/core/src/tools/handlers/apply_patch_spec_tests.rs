use super::*;
use codex_apply_patch::ApplyPatchArgs;
use codex_apply_patch::Hunk;
use codex_apply_patch::UpdateFileChunk;
use codex_apply_patch::parse_patch;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

#[test]
fn create_apply_patch_freeform_tool_matches_expected_spec() {
    assert_eq!(
        create_apply_patch_freeform_tool(/*include_environment_id*/ false),
        ToolSpec::Freeform(FreeformTool {
            name: "apply_patch".to_string(),
            description:
                "The `apply_patch` tool can be used to edit files. This is a FREEFORM tool, so do not wrap the patch in JSON."
                    .to_string(),
            defer_loading: None,
            format: FreeformToolFormat {
                r#type: "grammar".to_string(),
                syntax: "lark".to_string(),
                definition: APPLY_PATCH_LARK_GRAMMAR.to_string(),
            },
        })
    );
}

#[test]
fn create_apply_patch_freeform_tool_includes_environment_id_when_requested() {
    let ToolSpec::Freeform(tool) =
        create_apply_patch_freeform_tool(/*include_environment_id*/ true)
    else {
        panic!("expected freeform tool");
    };

    assert!(tool.format.definition.contains("environment_id?"));
    assert!(
        tool.format
            .definition
            .contains("\"*** Environment ID: \" filename LF")
    );
}

#[test]
fn create_apply_patch_function_tool_matches_expected_spec() {
    assert_eq!(
        create_apply_patch_function_tool(/*include_environment_id*/ false),
        ToolSpec::Function(ResponsesApiTool {
            name: "apply_patch".to_string(),
            description: APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION.to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                BTreeMap::from([(
                    "patch".to_string(),
                    JsonSchema::string(Some(
                        APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION.to_string(),
                    )),
                )]),
                Some(vec!["patch".to_string()]),
                Some(false.into()),
            ),
            output_schema: None,
        })
    );
}

#[test]
fn create_apply_patch_function_tool_includes_environment_id_when_requested() {
    let ToolSpec::Function(tool) =
        create_apply_patch_function_tool(/*include_environment_id*/ true)
    else {
        panic!("expected function tool");
    };
    let properties = tool.parameters.properties.as_ref().expect("properties");
    assert!(properties.contains_key("patch"));
    assert!(properties.contains_key("environment_id"));
    assert_eq!(
        tool.parameters.required.as_deref(),
        Some(["patch".to_string()].as_slice())
    );
}

#[test]
fn create_apply_patch_function_tool_teaches_patch_format_in_argument_description() {
    let ToolSpec::Function(tool) =
        create_apply_patch_function_tool(/*include_environment_id*/ false)
    else {
        panic!("expected function tool");
    };
    let properties = tool.parameters.properties.as_ref().expect("properties");
    let description = properties
        .get("patch")
        .and_then(|schema| schema.description.as_deref())
        .expect("patch argument description");
    for substring in [
        "first line is `*** Begin Patch`",
        "real newline characters",
        "bare '+'",
        "at most one hunk per patch",
        "starts with '+'",
        "multiple `@@` chunks",
        "must change at least one line",
    ] {
        assert!(
            description.contains(substring),
            "patch argument description missing: {substring}"
        );
    }
}

#[test]
fn create_apply_patch_function_tool_example_round_trips_through_parser() {
    let ToolSpec::Function(tool) =
        create_apply_patch_function_tool(/*include_environment_id*/ false)
    else {
        panic!("expected function tool");
    };
    let properties = tool.parameters.properties.as_ref().expect("properties");
    let description = properties
        .get("patch")
        .and_then(|schema| schema.description.as_deref())
        .expect("patch argument description");
    // Pinned extraction rule (spec §4 T2.2): split at the first line that is
    // exactly `Example:`; a naive substring search is wrong because the
    // first sentence contains the same phrase.
    let lines: Vec<&str> = description.lines().collect();
    let example_start = lines
        .iter()
        .position(|line| *line == "Example:")
        .expect("example section in patch argument description");
    let example = lines[example_start + 1..].join("\n");
    assert_eq!(
        example.lines().last().expect("example is non-empty"),
        "*** End Patch"
    );
    assert_eq!(
        parse_patch(&example),
        Ok(ApplyPatchArgs {
            patch: example.clone(),
            hunks: vec![
                Hunk::AddFile {
                    path: PathBuf::from("notes/todo.md"),
                    contents: "# TODO\n\n1. ship the fix\n".to_string(),
                },
                Hunk::UpdateFile {
                    path: PathBuf::from("src/main.rs"),
                    move_path: None,
                    chunks: vec![UpdateFileChunk {
                        change_context: Some("fn main".to_string()),
                        old_lines: vec!["    old_call();".to_string(), "    shared();".to_string(),],
                        new_lines: vec!["    new_call();".to_string(), "    shared();".to_string(),],
                        context_line_indices: vec![(1, 1)],
                        is_end_of_file: false,
                    }],
                },
            ],
            workdir: None,
            environment_id: None,
        })
    );
}
