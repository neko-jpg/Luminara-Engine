//! Operation timeline for persistent undo/redo with Git-like branch management
//!
//! This module provides a persistent operation timeline that stores all operations
//! with their inverse commands, enabling undo/redo functionality that persists
//! across sessions. It also supports Git-like branch management for experimental
//! changes.

use crate::error::{DbError, DbResult};
use crate::schema::OperationRecord;
use crate::LuminaraDatabase;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use surrealdb::RecordId;

/// Operation timeline manager
///
/// Manages a persistent timeline of operations with undo/redo support
/// and Git-like branch management.
pub struct OperationTimeline {
    /// Reference to the database
    db: LuminaraDatabase,
    /// Current branch name
    current_branch: String,
    /// Current position in the timeline (for undo/redo)
    current_position: Option<RecordId>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Head operation (most recent)
    pub head: Option<RecordId>,
    /// Number of operations in this branch
    pub operation_count: usize,
    /// Creation timestamp
    pub created_at: i64,
}

/// Timeline statistics
#[derive(Debug, Clone)]
pub struct TimelineStatistics {
    /// Total number of operations
    pub total_operations: usize,
    /// Number of operations in current branch
    pub branch_operations: usize,
    /// Current branch name
    pub current_branch: String,
    /// Number of operations that can be undone
    pub undoable_operations: usize,
    /// Number of operations that can be redone
    pub redoable_operations: usize,
}

impl OperationTimeline {
    /// Create a new operation timeline
    ///
    /// # Arguments
    ///
    /// * `db` - Database instance
    /// * `branch` - Initial branch name (defaults to "main")
    pub fn new(db: LuminaraDatabase, branch: Option<String>) -> Self {
        Self {
            db,
            current_branch: branch.unwrap_or_else(|| "main".to_string()),
            current_position: None,
        }
    }

    /// Record a new operation to the timeline
    ///
    /// # Arguments
    ///
    /// * `operation_type` - Type of operation (e.g., "SpawnEntity", "ModifyComponent")
    /// * `description` - Human-readable description
    /// * `commands` - Forward commands to execute
    /// * `inverse_commands` - Inverse commands for undo
    /// * `affected_entities` - Entities affected by this operation
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::{LuminaraDatabase, timeline::OperationTimeline};
    /// # use serde_json::json;
    /// # async fn example(timeline: &mut OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// let op_id = timeline.record_operation(
    ///     "SpawnEntity",
    ///     "Spawned player entity",
    ///     vec![json!({"type": "spawn", "entity": "player"})],
    ///     vec![json!({"type": "despawn", "entity": "player"})],
    ///     vec![],
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record_operation(
        &mut self,
        operation_type: impl Into<String>,
        description: impl Into<String>,
        commands: Vec<serde_json::Value>,
        inverse_commands: Vec<serde_json::Value>,
        affected_entities: Vec<RecordId>,
    ) -> DbResult<RecordId> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut operation = OperationRecord::new(
            operation_type,
            description,
            commands,
            inverse_commands,
            timestamp,
        )
        .with_branch(&self.current_branch);

        // Set parent to current position
        if let Some(parent_id) = &self.current_position {
            operation.parent = Some(parent_id.clone());
        }

        // Add affected entities
        for entity_id in affected_entities {
            operation = operation.with_affected_entity(entity_id);
        }

        // Store operation
        let operation_id = self.db.store_operation(operation).await?;

        // Update current position
        self.current_position = Some(operation_id.clone());

        Ok(operation_id)
    }

    /// Undo the last operation
    ///
    /// Returns the inverse commands that should be executed to undo the operation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &mut OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// if let Some((op_id, inverse_commands)) = timeline.undo().await? {
    ///     println!("Undoing operation: {:?}", op_id);
    ///     // Execute inverse commands...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn undo(&mut self) -> DbResult<Option<(RecordId, Vec<serde_json::Value>)>> {
        // Get current operation
        let current_id = match &self.current_position {
            Some(id) => id.clone(),
            None => return Ok(None), // Nothing to undo
        };

        // Load the operation
        let operation = self.db.load_operation(&current_id).await?;

        // Move position to parent
        self.current_position = operation.parent.clone();

        // Return operation ID and inverse commands
        Ok(Some((current_id, operation.inverse_commands)))
    }

    /// Redo the next operation
    ///
    /// Returns the forward commands that should be executed to redo the operation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &mut OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// if let Some((op_id, commands)) = timeline.redo().await? {
    ///     println!("Redoing operation: {:?}", op_id);
    ///     // Execute forward commands...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn redo(&mut self) -> DbResult<Option<(RecordId, Vec<serde_json::Value>)>> {
        // Find the next operation (child of current position)
        let next_operation = self.find_next_operation().await?;

        match next_operation {
            Some((op_id, operation)) => {
                // Move position forward
                self.current_position = Some(op_id.clone());

                // Return operation ID and forward commands
                Ok(Some((op_id, operation.commands)))
            }
            None => Ok(None), // Nothing to redo
        }
    }

    /// Find the next operation to redo
    async fn find_next_operation(&self) -> DbResult<Option<(RecordId, OperationRecord)>> {
        let parent_id = match &self.current_position {
            Some(id) => id,
            None => {
                // At the beginning, find the first operation in this branch
                let operations = self
                    .db
                    .load_operation_history(1, Some(&self.current_branch))
                    .await?;

                return Ok(operations
                    .into_iter()
                    .next()
                    .and_then(|op| op.id.clone().map(|id| (id, op))));
            }
        };

        // Find operations that have this as parent
        let query = format!(
            "SELECT * FROM operation WHERE parent = {} AND branch = '{}' ORDER BY timestamp ASC LIMIT 1",
            parent_id, self.current_branch
        );

        let mut result = self.db.execute_query(&query).await?;
        let operations: Vec<OperationRecord> = result.take(0)?;

        Ok(operations
            .into_iter()
            .next()
            .and_then(|op| op.id.clone().map(|id| (id, op))))
    }

    /// Get operation history for the current branch
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of operations to retrieve
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// let history = timeline.get_history(100).await?;
    /// println!("Found {} operations in history", history.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_history(&self, limit: usize) -> DbResult<Vec<OperationRecord>> {
        self.db
            .load_operation_history(limit, Some(&self.current_branch))
            .await
    }

    /// Get all operations (across all branches)
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of operations to retrieve
    pub async fn get_all_operations(&self, limit: usize) -> DbResult<Vec<OperationRecord>> {
        self.db.load_operation_history(limit, None).await
    }

    /// Create a new branch from the current position
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name for the new branch
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &mut OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// timeline.create_branch("experimental-feature").await?;
    /// println!("Created new branch, now on: {}", timeline.current_branch());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_branch(&mut self, branch_name: impl Into<String>) -> DbResult<()> {
        let branch_name = branch_name.into();

        // Check if branch already exists
        let existing = self.get_branch_info(&branch_name).await?;
        if existing.is_some() {
            return Err(DbError::Other(format!(
                "Branch '{}' already exists",
                branch_name
            )));
        }

        // Switch to the new branch
        self.current_branch = branch_name;

        Ok(())
    }

    /// Switch to a different branch
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch to switch to
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &mut OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// timeline.switch_branch("main").await?;
    /// println!("Switched to branch: {}", timeline.current_branch());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn switch_branch(&mut self, branch_name: impl Into<String>) -> DbResult<()> {
        let branch_name = branch_name.into();

        // Get branch info to verify it exists
        let branch_info = self.get_branch_info(&branch_name).await?;

        // Switch to the branch
        self.current_branch = branch_name;

        // Set position to the head of the branch
        self.current_position = branch_info.and_then(|info| info.head);

        Ok(())
    }

    /// Get information about a branch
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch
    pub async fn get_branch_info(&self, branch_name: &str) -> DbResult<Option<BranchInfo>> {
        // Query operations in this branch
        let operations = self
            .db
            .load_operation_history(1000, Some(branch_name))
            .await?;

        if operations.is_empty() {
            return Ok(None);
        }

        // Find the head (most recent operation)
        let head = operations.first().and_then(|op| op.id.clone());

        // Find creation time (oldest operation)
        let created_at = operations.last().map(|op| op.timestamp).unwrap_or(0);

        Ok(Some(BranchInfo {
            name: branch_name.to_string(),
            head,
            operation_count: operations.len(),
            created_at,
        }))
    }

    /// List all branches
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// let branches = timeline.list_branches().await?;
    /// for branch in branches {
    ///     println!("Branch: {} ({} operations)", branch.name, branch.operation_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_branches(&self) -> DbResult<Vec<BranchInfo>> {
        // Query all unique branch names
        let query = "SELECT DISTINCT branch FROM operation WHERE branch IS NOT NULL";
        let mut result = self.db.execute_query(query).await?;
        let branch_records: Vec<BranchRecord> = result.take(0)?;

        // Get info for each branch
        let mut branches = Vec::new();
        for record in branch_records {
            if let Some(branch_name) = record.branch {
                if let Some(info) = self.get_branch_info(&branch_name).await? {
                    branches.push(info);
                }
            }
        }

        Ok(branches)
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> &str {
        &self.current_branch
    }

    /// Get the current position in the timeline
    pub fn current_position(&self) -> Option<&RecordId> {
        self.current_position.as_ref()
    }

    /// Get timeline statistics
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use luminara_db::timeline::OperationTimeline;
    /// # async fn example(timeline: &OperationTimeline) -> Result<(), Box<dyn std::error::Error>> {
    /// let stats = timeline.get_statistics().await?;
    /// println!("Timeline has {} total operations", stats.total_operations);
    /// println!("Can undo {} operations", stats.undoable_operations);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_statistics(&self) -> DbResult<TimelineStatistics> {
        // Get all operations
        let all_operations = self.get_all_operations(10000).await?;
        let total_operations = all_operations.len();

        // Get branch operations
        let branch_operations = self.get_history(10000).await?;
        let branch_operation_count = branch_operations.len();

        // Count undoable operations (from current position back to root)
        let undoable_operations = self.count_undoable_operations().await?;

        // Count redoable operations (from current position forward)
        let redoable_operations = self.count_redoable_operations().await?;

        Ok(TimelineStatistics {
            total_operations,
            branch_operations: branch_operation_count,
            current_branch: self.current_branch.clone(),
            undoable_operations,
            redoable_operations,
        })
    }

    /// Count operations that can be undone
    async fn count_undoable_operations(&self) -> DbResult<usize> {
        let mut count = 0;
        let mut current = self.current_position.clone();

        while let Some(op_id) = current {
            count += 1;
            let operation = self.db.load_operation(&op_id).await?;
            current = operation.parent;
        }

        Ok(count)
    }

    /// Count operations that can be redone
    async fn count_redoable_operations(&self) -> DbResult<usize> {
        let mut count = 0;
        let mut current = self.current_position.clone();

        loop {
            // Find next operation
            let parent_id = match &current {
                Some(id) => id,
                None => break,
            };

            let query = format!(
                "SELECT * FROM operation WHERE parent = {} AND branch = '{}' LIMIT 1",
                parent_id, self.current_branch
            );

            let mut result = self.db.execute_query(&query).await?;
            let operations: Vec<OperationRecord> = result.take(0)?;

            if let Some(op) = operations.first() {
                count += 1;
                current = op.id.clone();
            } else {
                break;
            }
        }

        Ok(count)
    }

    /// Clear all operations in the current branch
    ///
    /// **Warning:** This is destructive and cannot be undone!
    pub async fn clear_branch(&mut self) -> DbResult<()> {
        let operations = self.get_history(10000).await?;

        for operation in operations {
            if let Some(op_id) = operation.id {
                self.db.delete_operation(&op_id).await?;
            }
        }

        self.current_position = None;

        Ok(())
    }

    /// Delete a branch
    ///
    /// **Warning:** This is destructive and cannot be undone!
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch to delete
    pub async fn delete_branch(&self, branch_name: &str) -> DbResult<()> {
        if branch_name == self.current_branch {
            return Err(DbError::Other(
                "Cannot delete the current branch".to_string(),
            ));
        }

        let operations = self
            .db
            .load_operation_history(10000, Some(branch_name))
            .await?;

        for operation in operations {
            if let Some(op_id) = operation.id {
                self.db.delete_operation(&op_id).await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct BranchRecord {
    branch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_undo_operation() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let mut timeline = OperationTimeline::new(db, None);

        // Record an operation
        let op_id = timeline
            .record_operation(
                "TestOp",
                "Test operation",
                vec![serde_json::json!({"action": "forward"})],
                vec![serde_json::json!({"action": "backward"})],
                vec![],
            )
            .await
            .unwrap();

        assert!(timeline.current_position().is_some());

        // Undo the operation
        let undo_result = timeline.undo().await.unwrap();
        assert!(undo_result.is_some());

        let (undone_id, inverse_commands) = undo_result.unwrap();
        assert_eq!(undone_id, op_id);
        assert_eq!(inverse_commands.len(), 1);
        assert_eq!(inverse_commands[0]["action"], "backward");

        // Position should be None after undo
        assert!(timeline.current_position().is_none());
    }

    #[tokio::test]
    async fn test_redo_operation() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let mut timeline = OperationTimeline::new(db, None);

        // Record an operation
        timeline
            .record_operation(
                "TestOp",
                "Test operation",
                vec![serde_json::json!({"action": "forward"})],
                vec![serde_json::json!({"action": "backward"})],
                vec![],
            )
            .await
            .unwrap();

        // Undo
        timeline.undo().await.unwrap();

        // Redo
        let redo_result = timeline.redo().await.unwrap();
        assert!(redo_result.is_some());

        let (_op_id, commands) = redo_result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0]["action"], "forward");
    }

    #[tokio::test]
    async fn test_branch_creation() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let mut timeline = OperationTimeline::new(db, None);

        // Record operation on main branch
        timeline
            .record_operation(
                "MainOp",
                "Main branch operation",
                vec![serde_json::json!({"branch": "main"})],
                vec![],
                vec![],
            )
            .await
            .unwrap();

        // Create new branch
        timeline.create_branch("feature").await.unwrap();
        assert_eq!(timeline.current_branch(), "feature");

        // Record operation on feature branch
        timeline
            .record_operation(
                "FeatureOp",
                "Feature branch operation",
                vec![serde_json::json!({"branch": "feature"})],
                vec![],
                vec![],
            )
            .await
            .unwrap();

        // List branches
        let branches = timeline.list_branches().await.unwrap();
        assert!(branches.iter().any(|b| b.name == "main"));
        assert!(branches.iter().any(|b| b.name == "feature"));
    }

    #[tokio::test]
    async fn test_branch_switching() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let mut timeline = OperationTimeline::new(db, None);

        // Record on main
        timeline
            .record_operation("Op1", "Operation 1", vec![], vec![], vec![])
            .await
            .unwrap();

        // Create and switch to feature branch
        timeline.create_branch("feature").await.unwrap();
        timeline
            .record_operation("Op2", "Operation 2", vec![], vec![], vec![])
            .await
            .unwrap();

        // Switch back to main
        timeline.switch_branch("main").await.unwrap();
        assert_eq!(timeline.current_branch(), "main");

        // History should only show main operations
        let history = timeline.get_history(10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].operation_type, "Op1");
    }

    #[tokio::test]
    async fn test_timeline_statistics() {
        let db = LuminaraDatabase::new_memory().await.unwrap();
        let mut timeline = OperationTimeline::new(db, None);

        // Record multiple operations
        for i in 0..5 {
            timeline
                .record_operation(
                    format!("Op{}", i),
                    format!("Operation {}", i),
                    vec![],
                    vec![],
                    vec![],
                )
                .await
                .unwrap();
        }

        let stats = timeline.get_statistics().await.unwrap();
        assert_eq!(stats.total_operations, 5);
        assert_eq!(stats.branch_operations, 5);
        assert_eq!(stats.undoable_operations, 5);
        assert_eq!(stats.redoable_operations, 0);

        // Undo 2 operations
        timeline.undo().await.unwrap();
        timeline.undo().await.unwrap();

        let stats = timeline.get_statistics().await.unwrap();
        assert_eq!(stats.undoable_operations, 3);
        assert_eq!(stats.redoable_operations, 2);
    }
}
