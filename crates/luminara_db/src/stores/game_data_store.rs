use crate::{connection::DbConnection, error::DbError};
use serde::de::DeserializeOwned;

pub struct GameDataStore<'a> {
    conn: &'a DbConnection,
}

impl<'a> GameDataStore<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    pub async fn get_table<T: DeserializeOwned>(&self, table_name: &str) -> Result<Vec<T>, DbError> {
        self.conn.use_database("game_data").await?;
        let mut result = self.conn.inner()
            .query("SELECT * FROM type::table($table)")
            .bind(("table", table_name.to_string()))
            .await?;
        let items: Vec<T> = result.take(0)?;
        Ok(items)
    }

    pub async fn get_by_id<T: DeserializeOwned>(&self, table_name: &str, id: &str) -> Result<Option<T>, DbError> {
        self.conn.use_database("game_data").await?;
        // Construct Thing manually or query by string ID if stored that way.
        // Assuming ID is "table:id" string in DB.
        let thing = format!("{}:{}", table_name, id);
        let mut result = self.conn.inner()
            .query("SELECT * FROM type::thing($thing)")
            .bind(("thing", thing))
            .await?;
        let item: Option<T> = result.take(0)?;
        Ok(item)
    }

    pub async fn query<T: DeserializeOwned>(&self, surql: &str) -> Result<Vec<T>, DbError> {
        self.conn.use_database("game_data").await?;
        let mut result = self.conn.inner().query(surql).await?;
        let items: Vec<T> = result.take(0)?;
        Ok(items)
    }
}
