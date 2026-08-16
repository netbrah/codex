use crate::tool_name::ToolName;

#[test]
fn from_response_fields_splits_flattened_dotted_name() {
    let tool_name = ToolName::from_response_fields(None, "collaboration.spawn_agent".to_string());
    assert_eq!(tool_name.namespace.as_deref(), Some("collaboration"));
    assert_eq!(tool_name.name, "spawn_agent");
}

#[test]
fn from_response_fields_preserves_explicit_namespace() {
    let tool_name = ToolName::from_response_fields(
        Some("collaboration".to_string()),
        "spawn_agent".to_string(),
    );
    assert_eq!(tool_name.namespace.as_deref(), Some("collaboration"));
    assert_eq!(tool_name.name, "spawn_agent");
}

#[test]
fn from_response_fields_plain_name_gets_default_namespace() {
    let tool_name = ToolName::from_response_fields(None, "exec_command".to_string());
    assert_eq!(tool_name.namespace.as_deref(), Some("functions"));
    assert_eq!(tool_name.name, "exec_command");
}
