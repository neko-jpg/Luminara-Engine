use luminara_mcp_server::{LuminaraMcpServer, McpTool, McpError, McpRequest};
use serde_json::{json, Value};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

struct EchoTool;
impl McpTool for EchoTool {
    fn name(&self) -> &str { "echo" }
    fn description(&self) -> &str { "Echoes input" }
    fn call(&self, params: Value) -> Result<Value, McpError> {
        Ok(params)
    }
    fn input_schema(&self) -> Value { json!({}) }
}

#[test]
fn test_mcp_request_handling() {
    let mut server = LuminaraMcpServer::new();
    server.register_tool(Box::new(EchoTool));

    let req = McpRequest {
        jsonrpc: "2.0".into(),
        id: Some(1),
        method: "list_tools".into(),
        params: None,
    };
    let resp = server.handle_request(req);
    assert!(resp.result.is_some());

    let req = McpRequest {
        jsonrpc: "2.0".into(),
        id: Some(2),
        method: "call_tool".into(),
        params: Some(json!({
            "name": "echo",
            "arguments": { "msg": "hello" }
        })),
    };
    let resp = server.handle_request(req);
    assert!(resp.result.is_some());
    assert_eq!(resp.result.unwrap(), json!({ "msg": "hello" }));
}

#[quickcheck]
fn test_mcp_invalid_tool_name(name: String) -> TestResult {
    if name == "echo" { return TestResult::discard(); }

    let mut server = LuminaraMcpServer::new();
    server.register_tool(Box::new(EchoTool));

    let req = McpRequest {
        jsonrpc: "2.0".into(),
        id: Some(1),
        method: "call_tool".into(),
        params: Some(json!({
            "name": name,
            "arguments": {}
        })),
    };

    let resp = server.handle_request(req);
    TestResult::from_bool(resp.error.is_some())
}
