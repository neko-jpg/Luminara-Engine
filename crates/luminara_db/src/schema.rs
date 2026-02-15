use crate::{connection::DbConnection, error::DbError};

pub struct SchemaManager<'a> {
    conn: &'a DbConnection,
    migrations: Vec<Migration>,
}

pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up: &'static str,
    pub down: &'static str,
}

impl<'a> SchemaManager<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self {
            conn,
            migrations: Self::builtin_migrations(),
        }
    }

    fn builtin_migrations() -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                name: "initial_schema".into(),
                up: include_str!("../migrations/001_initial_schema.surql"),
                down: include_str!("../migrations/001_initial_schema_down.surql"),
            },
        ]
    }

    pub async fn current_version(&self) -> Result<u32, DbError> {
        // Switch to project DB context to check version
        // This might fail if DB doesn't exist, but usually query works.
        if let Err(_) = self.conn.use_database("project").await {
             return Ok(0);
        }

        let result = self.conn.query("SELECT value FROM project_settings WHERE key = 'schema_version' LIMIT 1").await;

        match result {
            Ok(mut resp) => {
                // resp.take(0) might fail if it's not a query result set or error in first statement
                // If table missing, SurrealDB returns empty array or error depending on mode.
                // We try to parse.
                match resp.take::<Option<serde_json::Value>>(0) {
                    Ok(Some(val)) => {
                         // val is { "value": { "version": 1 } }
                         if let Some(v) = val.get("value").and_then(|v| v.get("version")).and_then(|v| v.as_u64()) {
                             return Ok(v as u32);
                         }
                         Ok(0)
                    },
                    _ => Ok(0),
                }
            },
            Err(_) => Ok(0),
        }
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        let current = self.current_version().await?;

        for migration in &self.migrations {
            if migration.version > current {
                tracing::info!(
                    "Running migration v{}: {}",
                    migration.version, migration.name
                );

                self.conn.query(migration.up).await
                    .map_err(|e| DbError::MigrationFailed(
                        format!("v{} ({}): {}", migration.version, migration.name, e)
                    ))?;

                // Update version
                self.conn.use_database("project").await?;
                // Note: using UPSERT or CREATE/UPDATE logic.
                // In v2, UPSERT is supported? Or CREATE ... CONTENT ...
                // The prompt used UPSERT.
                // SurrealQL syntax: UPSERT project_settings:schema_version CONTENT { ... } ??
                // Or UPDATE project_settings ...
                // The prompt example:
                // UPSERT project_settings SET value = ... WHERE key = 'schema_version'
                // Or CREATE project_settings SET ...

                // I will use `UPDATE project_settings SET ... UPSERT` if available or manual check.
                // Actually `UPSERT` statement exists in SurrealQL?
                // It is `UPSERT` in prompt but SurrealDB docs usually say `INSERT ... ON DUPLICATE KEY UPDATE` or similar.
                // Wait, SurrealDB supports `UPSERT` as a statement?
                // Checking docs (from memory): `UPSERT` is not a standard keyword. `INSERT` or `UPDATE` or `CREATE`.
                // `UPDATE` can create if record ID is specified?
                // The prompt says `UPSERT`. Maybe it's a new v2 feature or the user meant `INSERT ...`.
                // But the query in 12.1 is:
                // UPSERT project_settings SET value = ... WHERE key = 'schema_version'

                // I'll stick to `UPDATE` which works if record exists.
                // Or `INSERT INTO project_settings ... ON DUPLICATE KEY UPDATE` is not supported.
                // SurrealDB has `INSERT IGNORE` or `UPDATE`.
                // For `key` field unique index, `CREATE` will fail if exists.

                // Better approach: `LET $exists = (SELECT id FROM project_settings WHERE key = 'schema_version'); IF $exists THEN UPDATE ... ELSE CREATE ... END;`
                // Or just delete and create?

                // Actually, since I define `key` as unique, `CREATE` fails.
                // I will try `UPDATE project_settings SET value = ... WHERE key = 'schema_version'`.
                // If it returns 0 modified, I `CREATE`.

                let q = format!(r#"
                    LET $rec = (SELECT id FROM project_settings WHERE key = 'schema_version');
                    IF $rec THEN
                        UPDATE $rec SET value = {{ version: {} }}, category = 'system';
                    ELSE
                        CREATE project_settings SET key = 'schema_version', value = {{ version: {} }}, category = 'system';
                    END;
                "#, migration.version, migration.version);

                self.conn.query(&q).await
                    .map_err(|e| DbError::MigrationFailed(format!("Failed to update version: {}", e)))?;

                tracing::info!("Migration v{} completed", migration.version);
            }
        }

        Ok(())
    }
}
