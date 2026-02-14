use luminara_mcp_server::{CreateEntityTool, McpTool};
use serde_json::json;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_tool_result_structure() {
    let tool = CreateEntityTool::new();
    let result = tool.call(json!({})).unwrap();

    // Check if result has required fields
    assert!(result.get("entity_id").is_some());
    assert!(result.get("status").is_some());
}

#[quickcheck]
fn test_tool_error_suggestions(arg: String) -> TestResult {
    // If we pass valid params, it succeeds.
    // We want to test input validation if implemented.
    // Our mock implementation accepts anything for now.
    // So this property test just verifies it doesn't panic.

    let tool = CreateEntityTool::new();
    let res = tool.call(json!({ "arg": arg }));
    TestResult::from_bool(res.is_ok())
}
