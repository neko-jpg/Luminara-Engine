use crate::{connection::DbConnection, models::save_game::*, error::DbError};
use chrono::Utc;
use surrealdb::sql::Datetime;

pub struct SaveStore<'a> {
    conn: &'a DbConnection,
}

impl<'a> SaveStore<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    pub async fn create_slot(&self, slot: &SaveSlot) -> Result<(), DbError> {
        self.conn.use_database("saves").await?;
        self.conn.inner()
            .create::<Option<SaveSlot>>("save_slot")
            .content(slot.clone())
            .await?;
        Ok(())
    }

    pub async fn list_slots(&self) -> Result<Vec<SaveSlot>, DbError> {
        self.conn.use_database("saves").await?;
        let mut result = self.conn.inner()
            .query("SELECT * FROM save_slot ORDER BY updated_at DESC")
            .await?;
        let slots: Vec<SaveSlot> = result.take(0)?;
        Ok(slots)
    }

    pub async fn load_slot(&self, name: &str) -> Result<SaveSlot, DbError> {
        self.conn.use_database("saves").await?;
        let mut result = self.conn.inner()
            .query("SELECT * FROM save_slot WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await?;
        let slot: SaveSlot = result.take::<Option<SaveSlot>>(0)?
            .ok_or(DbError::InvalidData(format!("Save slot '{}' not found", name)))?;
        Ok(slot)
    }

    pub async fn delete_slot(&self, name: &str) -> Result<(), DbError> {
        self.conn.use_database("saves").await?;
        self.conn.inner()
            .query("DELETE save_slot WHERE name = $name")
            .bind(("name", name.to_string()))
            .await?;
        Ok(())
    }

    pub async fn update_play_time(&self, name: &str, duration: std::time::Duration) -> Result<(), DbError> {
        self.conn.use_database("saves").await?;
        self.conn.inner()
            .query("UPDATE save_slot SET play_time = $time, updated_at = $now WHERE name = $name")
            .bind(("name", name.to_string()))
            .bind(("time", duration))
            .bind(("now", Datetime::from(Utc::now())))
            .await?;
        Ok(())
    }
}
