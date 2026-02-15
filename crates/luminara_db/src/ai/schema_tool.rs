use luminara_mcp::{McpTool, McpError};
use crate::sync::commands::DbCommandSender;
use serde_json::Value;

pub struct DbSchemaTool {
    pub sender: DbCommandSender,
}

impl McpTool for DbSchemaTool {
    fn name(&self) -> &str { "db.schema" }

    fn description(&self) -> &str {
        "Retrieves schema information for the database or a specific table."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "database": {
                    "type": "string",
                    "description": "Target database",
                    "enum": ["assets", "scenes", "game_data", "saves", "history"]
                },
                "table": {
                    "type": "string",
                    "description": "Optional table name to inspect. If omitted, lists all tables."
                }
            },
            "required": ["database"]
        })
    }

    fn call(&self, input: Value) -> Result<Value, McpError> {
        let database = input["database"].as_str().unwrap_or("scenes");

        let query = if let Some(table) = input.get("table").and_then(|v| v.as_str()) {
            format!("INFO FOR TABLE {}", table)
        } else {
            "INFO FOR DB".to_string()
        };

        let full_query = format!("USE DB {}; {}", database, query);
        let rx = self.sender.query(full_query);

        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            rx.await.map_err(|_| McpError::Internal("DB channel closed".into()))?
                .map_err(|e| McpError::Internal(format!("Schema query error: {}", e)))
        })?;

        Ok(result)
    }
}
