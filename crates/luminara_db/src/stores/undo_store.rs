use crate::{connection::DbConnection, models::undo_meta::*, error::DbError};
use surrealdb::sql::Thing;
use serde::Deserialize;

pub struct UndoStore<'a> {
    conn: &'a DbConnection,
    max_history: usize,
}

impl<'a> UndoStore<'a> {
    pub fn new(conn: &'a DbConnection, max_history: usize) -> Self {
        Self { conn, max_history }
    }

    pub async fn push(&self, entry: &UndoEntry) -> Result<(), DbError> {
        self.conn.use_database("history").await?;

        self.conn.inner()
            .create::<Option<UndoEntry>>("undo_entry")
            .content(entry.clone())
            .await?;

        // Remove old entries: count first
        let mut count_resp = self.conn.inner()
            .query("SELECT count() FROM undo_entry GROUP ALL")
            .await?;

        #[derive(Deserialize)]
        struct Count { count: usize }
        // The result of SELECT count() ... is `[{ count: N }]`.
        let counts: Vec<Count> = count_resp.take(0)?;
        let count = counts.first().map(|c| c.count).unwrap_or(0);

        if count > self.max_history {
            let limit = count - self.max_history;

             let mut targets_resp = self.conn.inner()
                .query("SELECT id FROM undo_entry ORDER BY timestamp ASC LIMIT $limit")
                .bind(("limit", limit))
                .await?;

             #[derive(Deserialize)]
             struct IdRow { id: Thing }
             let rows: Vec<IdRow> = targets_resp.take(0)?;

             for row in rows {
                 self.conn.inner().delete::<Option<UndoEntry>>((row.id.tb, row.id.id.to_string())).await?;
             }
        }

        Ok(())
    }

    pub async fn peek_undo(&self) -> Result<Option<UndoEntry>, DbError> {
        self.conn.use_database("history").await?;

        let mut result = self.conn.inner()
            .query("SELECT * FROM undo_entry ORDER BY timestamp DESC LIMIT 1")
            .await?;
        let entry: Option<UndoEntry> = result.take(0)?;
        Ok(entry)
    }

    pub async fn pop_undo(&self) -> Result<Option<UndoEntry>, DbError> {
        self.conn.use_database("history").await?;

        let mut peek = self.conn.inner()
            .query("SELECT * FROM undo_entry ORDER BY timestamp DESC LIMIT 1")
            .await?;

        let entries: Vec<UndoEntry> = peek.take(0)?;
        if let Some(entry) = entries.first() {
            if let Some(id) = &entry.id {
                 self.conn.inner().delete::<Option<UndoEntry>>((id.tb.clone(), id.id.to_string())).await?;
                 return Ok(Some(entry.clone()));
            }
        }

        Ok(None)
    }

    pub async fn clear(&self) -> Result<(), DbError> {
        self.conn.use_database("history").await?;
        self.conn.inner().query("DELETE undo_entry").await?;
        Ok(())
    }
}
