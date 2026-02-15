use crate::{connection::DbConnection, models::asset_meta::*, error::DbError};
use uuid::Uuid;
use surrealdb::sql::{Thing, Uuid as DbUuid};
use serde::Deserialize;

pub struct AssetStore<'a> {
    conn: &'a DbConnection,
}

impl<'a> AssetStore<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    pub async fn register(&self, meta: &AssetMeta) -> Result<Thing, DbError> {
        self.conn.use_database("assets").await?;

        let result: Option<AssetMeta> = self.conn.inner()
            .create("asset")
            .content(meta.clone())
            .await?;

        result.ok_or(DbError::InvalidData("Failed to create asset".into()))
              .and_then(|a| a.id.ok_or(DbError::InvalidData("Created asset has no ID".into())))
    }

    pub async fn get_by_uuid(&self, uuid: &Uuid) -> Result<Option<AssetMeta>, DbError> {
        self.conn.use_database("assets").await?;

        let db_uuid = DbUuid::from(*uuid);

        let mut result = self.conn.inner()
            .query("SELECT * FROM asset WHERE uuid = $uuid LIMIT 1")
            .bind(("uuid", db_uuid))
            .await?;

        let asset: Option<AssetMeta> = result.take(0)?;
        Ok(asset)
    }

    pub async fn get_by_path(&self, path: &str) -> Result<Option<AssetMeta>, DbError> {
        self.conn.use_database("assets").await?;

        let mut result = self.conn.inner()
            .query("SELECT * FROM asset WHERE path = $path LIMIT 1")
            .bind(("path", path.to_string()))
            .await?;

        let asset: Option<AssetMeta> = result.take(0)?;
        Ok(asset)
    }

    pub async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<AssetMeta>, DbError> {
        self.conn.use_database("assets").await?;

        let mut result = self.conn.inner()
            .query("SELECT * FROM asset WHERE tags CONTAINSANY $tags ORDER BY updated_at DESC")
            .bind(("tags", tags.to_vec()))
            .await?;

        let assets: Vec<AssetMeta> = result.take(0)?;
        Ok(assets)
    }

    pub async fn list_by_type(&self, asset_type: AssetType) -> Result<Vec<AssetMeta>, DbError> {
        self.conn.use_database("assets").await?;

        let mut result = self.conn.inner()
            .query("SELECT * FROM asset WHERE asset_type = $asset_type ORDER BY path")
            .bind(("asset_type", asset_type))
            .await?;

        let assets: Vec<AssetMeta> = result.take(0)?;
        Ok(assets)
    }

    pub async fn add_dependency(
        &self,
        from_uuid: &Uuid,
        to_uuid: &Uuid,
        dep_type: &str,
    ) -> Result<(), DbError> {
        self.conn.use_database("assets").await?;

        let from_db_uuid = DbUuid::from(*from_uuid);
        let to_db_uuid = DbUuid::from(*to_uuid);

        let response = self.conn.inner()
            .query(r#"
                LET $from = (SELECT id FROM asset WHERE uuid = $from_uuid LIMIT 1);
                LET $to = (SELECT id FROM asset WHERE uuid = $to_uuid LIMIT 1);
                RELATE $from->depends_on->$to SET dependency_type = $dep_type;
            "#)
            .bind(("from_uuid", from_db_uuid))
            .bind(("to_uuid", to_db_uuid))
            .bind(("dep_type", dep_type.to_string()))
            .await?;

        response.check()?;
        Ok(())
    }

    pub async fn get_dependency_tree(&self, uuid: &Uuid) -> Result<Vec<AssetMeta>, DbError> {
        self.conn.use_database("assets").await?;

        let db_uuid = DbUuid::from(*uuid);

        let mut result = self.conn.inner()
            .query(r#"
                SELECT ->depends_on->asset.* AS deps
                FROM asset
                WHERE uuid = $uuid
                FETCH deps
            "#)
            .bind(("uuid", db_uuid))
            .await?;

        #[derive(Deserialize)]
        struct Row {
            deps: Vec<AssetMeta>,
        }

        let rows: Vec<Row> = result.take(0)?;
        if let Some(row) = rows.into_iter().next() {
            Ok(row.deps)
        } else {
            Ok(vec![])
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<AssetMeta>, DbError> {
        self.conn.use_database("assets").await?;

        let mut result = self.conn.inner()
            .query(r#"
                SELECT *, search::score(0) AS score
                FROM asset
                WHERE tags @0@ $query OR path CONTAINS $query
                ORDER BY score DESC
                LIMIT 50
            "#)
            .bind(("query", query.to_string()))
            .await?;

        let assets: Vec<AssetMeta> = result.take(0)?;
        Ok(assets)
    }

    pub async fn update(&self, uuid: &Uuid, meta: &AssetMeta) -> Result<(), DbError> {
        self.conn.use_database("assets").await?;

        let db_uuid = DbUuid::from(*uuid);

        let response = self.conn.inner()
            .query("UPDATE asset SET
                file_hash = $file_hash,
                processed_hash = $processed_hash,
                file_size = $file_size,
                updated_at = time::now(),
                metadata = $metadata,
                tags = $tags
                WHERE uuid = $uuid")
            .bind(("uuid", db_uuid))
            .bind(("file_hash", meta.file_hash.clone()))
            .bind(("processed_hash", meta.processed_hash.clone()))
            .bind(("file_size", meta.file_size))
            .bind(("metadata", meta.metadata.clone()))
            .bind(("tags", meta.tags.clone()))
            .await?;

        response.check()?;
        Ok(())
    }

    pub async fn delete(&self, uuid: &Uuid) -> Result<(), DbError> {
        self.conn.use_database("assets").await?;

        let db_uuid = DbUuid::from(*uuid);

        let response = self.conn.inner()
            .query(r#"
                LET $asset = (SELECT id FROM asset WHERE uuid = $uuid);
                DELETE $asset->depends_on;
                DELETE depends_on WHERE out = $asset;
                DELETE $asset;
            "#)
            .bind(("uuid", db_uuid))
            .await?;

        response.check()?;
        Ok(())
    }
}
