use std::collections::BTreeMap;

use codex_tools::FreeformTool;
use codex_tools::FreeformToolFormat;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;

const APPLY_PATCH_LARK_GRAMMAR: &str = include_str!("../../../assets/tools/apply_patch.lark");

/// Returns a custom tool that can be used to edit files. Well-suited for GPT-5 models
/// https://platform.openai.com/docs/guides/function-calling#custom-tools
pub fn create_apply_patch_freeform_tool(include_environment_id: bool) -> ToolSpec {
    let definition = if include_environment_id {
        APPLY_PATCH_LARK_GRAMMAR.replace(
            "start: begin_patch hunk+ end_patch",
            "start: begin_patch environment_id? hunk+ end_patch\nenvironment_id: \"*** Environment ID: \" filename LF",
        )
    } else {
        APPLY_PATCH_LARK_GRAMMAR.to_string()
    };

    ToolSpec::Freeform(FreeformTool {
        name: "apply_patch".to_string(),
        description: "The `apply_patch` tool can be used to edit files. This is a FREEFORM tool, so do not wrap the patch in JSON.".to_string(),
        defer_loading: None,
        format: FreeformToolFormat {
            r#type: "grammar".to_string(),
            syntax: "lark".to_string(),
            definition,
        },
    })
}

/// Tool-level description for the function-tool form of `apply_patch`,
/// verbatim from spec §3.1 P1 (drift-guarded in `apply_patch_spec_tests.rs`).
const APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION: &str = "The `apply_patch` tool can be used to edit files (add, delete, update, move). The complete patch goes in the `patch` argument.";

/// `patch` argument description for the function-tool form of
/// `apply_patch`; carries the full patch-format teaching, verbatim from
/// spec §3.1 P1 (drift-guarded in `apply_patch_spec_tests.rs`).
const APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION: &str = "The ENTIRE patch as a single string: the first line is `*** Begin Patch`, the last line is `*** End Patch`, and everything between is the patch body. Lines inside the string are separated by real newline characters — never by the two characters backslash + n. Do not add markdown fences, code-block markers, or a shell heredoc wrapper around the patch.

FORMAT — the patch is line-oriented; every file-content line carries a one-character prefix that is part of the patch, not part of the file content:

*** Add File: <path>          creates a new file; EVERY following content line starts with '+'
*** Delete File: <path>       deletes a file; takes no content lines
*** Update File: <path>       modifies a file; see hunks below
*** Move to: <path>           optional; renames the file in the current Update hunk
@@ [context line]             starts a change chunk; optional context pins the location
<chunk lines>                 each line starts with ' ' (context/unchanged), '-' (removed), or '+' (added)
*** End of File               optional; chunk extends to end of file

Rules:
- After `*** Add File:`, every line of the new file starts with '+'. An empty file line is a bare '+' with nothing after it. A file line that itself starts with '+' gets the prefix on top (file line `+42` → patch line `++42`). To create an empty file, write no content lines after the header.
- In Update hunks, prefix every line: ' ' + line for unchanged context, '-' + line for removal, '+' + line for addition. To change several places in one file, put multiple `@@` chunks inside the single `*** Update File:` hunk.
- Never write raw (unprefixed) file content lines anywhere in the patch.
- Each file may appear in at most one hunk per patch; do not target the same file twice (the tool rejects patches that do). An Update hunk must change at least one line; to rename a file without editing it, use `*** Delete File:` followed by `*** Add File:`.
- Multiple hunks for different files may appear in one patch, in any order.

Example:
*** Begin Patch
*** Add File: notes/todo.md
+# TODO
+
+1. ship the fix
*** Update File: src/main.rs
@@ fn main
-    old_call();
+    new_call();
     shared();
*** End Patch";

/// Returns the JSON-schema function-tool form of `apply_patch` for
/// deployments whose responses implementation does not support grammar
/// constraints on custom tools (e.g. vLLM behind a proxy). The model puts
/// the full freeform patch text in the `patch` argument; the harness parses
/// and executes it exactly like the custom-tool path.
pub fn create_apply_patch_function_tool(include_environment_id: bool) -> ToolSpec {
    let mut properties = BTreeMap::from([(
        "patch".to_string(),
        JsonSchema::string(Some(
            APPLY_PATCH_FUNCTION_PATCH_ARGUMENT_DESCRIPTION.to_string(),
        )),
    )]);
    if include_environment_id {
        properties.insert(
            "environment_id".to_string(),
            JsonSchema::string(Some(
                "Optional identifier selecting the environment the patch applies to.".to_string(),
            )),
        );
    }
    ToolSpec::Function(ResponsesApiTool {
        name: "apply_patch".to_string(),
        description: APPLY_PATCH_FUNCTION_TOOL_DESCRIPTION.to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["patch".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

#[cfg(test)]
#[path = "apply_patch_spec_tests.rs"]
mod tests;
