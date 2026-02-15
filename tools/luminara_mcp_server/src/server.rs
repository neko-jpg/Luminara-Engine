use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn call(&self, params: Value) -> Result<Value, McpError>;
    fn input_schema(&self) -> Value;
}

pub struct LuminaraMcpServer {
    tools: HashMap<String, Box<dyn McpTool>>,
}

impl LuminaraMcpServer {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn McpTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "list_tools" => {
                let tools = self
                    .tools
                    .values()
                    .map(|t| ToolDescription {
                        name: t.name().to_string(),
                        description: t.description().to_string(),
                        input_schema: t.input_schema(),
                    })
                    .collect::<Vec<_>>();
                McpResponse::success(request.id, serde_json::to_value(tools).unwrap())
            }
            "call_tool" => {
                // Parse params
                if let Some(params) = request.params {
                    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let args = params.get("arguments").cloned().unwrap_or(Value::Null);

                    if let Some(tool) = self.tools.get(name) {
                        match tool.call(args) {
                            Ok(result) => McpResponse::success(request.id, result),
                            Err(e) => McpResponse::error(request.id, -32602, e.to_string()),
                        }
                    } else {
                        McpResponse::error(request.id, -32601, format!("Tool '{}' not found", name))
                    }
                } else {
                    McpResponse::error(request.id, -32602, "Missing params".into())
                }
            }
            _ => McpResponse::error(request.id, -32601, "Method not found".into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<McpErrorResponse>,
}

impl McpResponse {
    pub fn success(id: Option<u64>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<u64>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(McpErrorResponse {
                code,
                message,
                data: None,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct McpErrorResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Serialize, Debug)]
struct ToolDescription {
    name: String,
    description: String,
    input_schema: Value,
}
