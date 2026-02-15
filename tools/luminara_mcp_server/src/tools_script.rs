use crate::server::{McpError, McpTool};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

pub struct CreateScriptTool;
impl McpTool for CreateScriptTool {
    fn name(&self) -> &str {
        "script.create"
    }
    fn description(&self) -> &str {
        "Creates a new script file"
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            }
        })
    }
    fn call(&self, params: Value) -> Result<Value, McpError> {
        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or(McpError::InvalidParams("Missing path".into()))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or(McpError::InvalidParams("Missing content".into()))?;

        let path = PathBuf::from(path_str);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| McpError::Internal(e.to_string()))?;
        }

        fs::write(&path, content).map_err(|e| McpError::Internal(e.to_string()))?;

        // Mock ID logic
        Ok(json!({ "script_id": 100, "path": path_str }))
    }
}

pub struct ModifyScriptTool;
impl McpTool for ModifyScriptTool {
    fn name(&self) -> &str {
        "script.modify"
    }
    fn description(&self) -> &str {
        "Modifies an existing script"
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            }
        })
    }
    fn call(&self, params: Value) -> Result<Value, McpError> {
        // Reuse create logic essentially, but maybe trigger reload?
        // Since we are mocking engine, just write file.
        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or(McpError::InvalidParams("Missing path".into()))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or(McpError::InvalidParams("Missing content".into()))?;

        fs::write(path_str, content).map_err(|e| McpError::Internal(e.to_string()))?;

        Ok(json!({ "status": "reloaded" }))
    }
}

pub struct DebugInspectTool;
impl McpTool for DebugInspectTool {
    fn name(&self) -> &str {
        "debug.inspect"
    }
    fn description(&self) -> &str {
        "Inspects world state"
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            }
        })
    }
    fn call(&self, _params: Value) -> Result<Value, McpError> {
        Ok(json!({ "entities": 42, "fps": 60.0 }))
    }
}

pub struct ProjectScaffoldTool;
impl McpTool for ProjectScaffoldTool {
    fn name(&self) -> &str {
        "project.scaffold"
    }
    fn description(&self) -> &str {
        "Scaffolds a new project"
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["name", "path"],
            "properties": {
                "name": { "type": "string" },
                "path": { "type": "string" }
            }
        })
    }
    fn call(&self, params: Value) -> Result<Value, McpError> {
        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or(McpError::InvalidParams("Missing path".into()))?;

        let root = PathBuf::from(path_str);
        fs::create_dir_all(root.join("assets/scripts"))
            .map_err(|e| McpError::Internal(e.to_string()))?;
        fs::create_dir_all(root.join("assets/scenes"))
            .map_err(|e| McpError::Internal(e.to_string()))?;
        fs::create_dir_all(root.join("src")).map_err(|e| McpError::Internal(e.to_string()))?;

        fs::write(root.join("Cargo.toml"), "[package]\nname = \"game\"\n")
            .map_err(|e| McpError::Internal(e.to_string()))?;

        Ok(json!({ "path": path_str, "status": "created" }))
    }
}
