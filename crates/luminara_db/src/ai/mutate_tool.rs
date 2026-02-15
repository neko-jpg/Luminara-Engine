use luminara_mcp::{McpTool, McpError};
use crate::sync::commands::DbCommandSender;
use crate::models::undo_meta::UndoEntry;
use serde_json::Value;
use chrono::Utc;
use surrealdb::sql::Datetime;

pub struct DbMutateTool {
    pub sender: DbCommandSender,
}

impl McpTool for DbMutateTool {
    fn name(&self) -> &str { "db.mutate" }

    fn description(&self) -> &str {
        "Executes SurrealQL queries to modify game data (INSERT, UPDATE, DELETE, RELATE). \
         Changes are recorded in the undo history."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "database": {
                    "type": "string",
                    "description": "Target database",
                    "enum": ["scenes", "game_data", "assets"]
                },
                "operation": {
                    "type": "string",
                    "description": "SurrealQL mutation query."
                },
                "description": {
                    "type": "string",
                    "description": "Description of the change for undo history."
                }
            },
            "required": ["database", "operation", "description"]
        })
    }

    fn call(&self, input: Value) -> Result<Value, McpError> {
        let database = input["database"].as_str().unwrap_or("scenes");
        let operation = input["operation"].as_str().ok_or(McpError::InvalidParams("Missing operation".into()))?;
        let description = input["description"].as_str().unwrap_or("Unknown mutation");

        // TODO: In a robust implementation, we would fetch the pre-mutation state for Undo.

        let full_query = format!("USE DB {}; {}", database, operation);
        let rx = self.sender.query(full_query);

        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            rx.await.map_err(|_| McpError::Internal("DB channel closed".into()))?
                .map_err(|e| McpError::Internal(format!("Mutation error: {}", e)))
        })?;

        // Record Undo Entry (Best effort)
        let entry = UndoEntry {
            id: None,
            group_id: uuid::Uuid::new_v4().to_string(),
            sequence: 0,
            command_type: "SurrealQL".into(),
            description: description.to_string(),
            forward_data: serde_json::json!({ "query": operation, "database": database }),
            backward_data: serde_json::Value::Null,
            timestamp: Datetime::from(Utc::now()),
        };
        self.sender.record_undo(entry);

        Ok(result)
    }
}
