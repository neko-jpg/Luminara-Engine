use crate::{connection::DbConnection, models::scene::*, error::DbError};
use serde::Deserialize;

pub struct SceneStore<'a> {
    conn: &'a DbConnection,
}

impl<'a> SceneStore<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    pub async fn save_scene(&self, snapshot: &SceneSnapshot) -> Result<(), DbError> {
        self.conn.use_database("scenes").await?;

        let mut query = String::from("BEGIN TRANSACTION;\n");

        query.push_str(&format!(
            "UPSERT scene:⟨{}⟩ CONTENT {};\n",
            snapshot.scene_id,
            serde_json::to_string(&snapshot.scene)?
        ));

        for entity in &snapshot.entities {
            let entity_id = entity.id.as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| format!("entity:⟨{}⟩", uuid::Uuid::new_v4()));

            query.push_str(&format!(
                "UPSERT {} CONTENT {};\n",
                entity_id,
                serde_json::to_string(entity)?
            ));
        }

        for component in &snapshot.components {
            let comp_id = component.id.as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| format!("component:⟨{}⟩", uuid::Uuid::new_v4()));

            query.push_str(&format!(
                "UPSERT {} CONTENT {};\n",
                comp_id,
                serde_json::to_string(component)?
            ));
        }

        for relation in &snapshot.hierarchy {
             query.push_str(&format!(
                "RELATE {}->parent_of->{} SET order = {};\n",
                relation.parent_db_id, relation.child_db_id, relation.order
            ));
        }

        query.push_str("COMMIT TRANSACTION;\n");

        let response = self.conn.query(&query).await?;
        response.check()?;

        tracing::info!("Scene '{}' saved ({} entities, {} components)",
            snapshot.scene.name,
            snapshot.entities.len(),
            snapshot.components.len(),
        );

        Ok(())
    }

    pub async fn load_scene(&self, scene_name: &str) -> Result<SceneSnapshot, DbError> {
        self.conn.use_database("scenes").await?;

        let mut result = self.conn.inner()
            .query("SELECT * FROM scene WHERE name = $name LIMIT 1")
            .bind(("name", scene_name.to_string()))
            .await?;

        let scene: SceneRecord = result.take::<Option<SceneRecord>>(0)?
            .ok_or(DbError::SceneNotFound(scene_name.to_string()))?;

        let scene_thing = scene.id.clone().ok_or(DbError::InvalidData("Scene record has no ID".into()))?;

        // Load entities
        let mut result = self.conn.inner()
            .query("SELECT * FROM entity WHERE scene = $scene_id ORDER BY order")
            .bind(("scene_id", scene_thing.clone()))
            .await?;
        let entities: Vec<EntityRecord> = result.take(0)?;

        // Load components
        let mut result = self.conn.inner()
            .query(r#"
                SELECT * FROM component
                WHERE entity.scene = $scene_id
                ORDER BY entity, component_type
            "#)
            .bind(("scene_id", scene_thing.clone()))
            .await?;
        let components: Vec<ComponentRecord> = result.take(0)?;

        // Load hierarchy
        let mut result = self.conn.inner()
            .query(r#"
                SELECT in AS parent, out AS child, order
                FROM parent_of
                WHERE in.scene = $scene_id
                ORDER BY order
            "#)
            .bind(("scene_id", scene_thing.clone()))
            .await?;

        #[derive(Deserialize)]
        struct RelationRow {
            parent: surrealdb::sql::Thing,
            child: surrealdb::sql::Thing,
            order: i32,
        }

        let rows: Vec<RelationRow> = result.take(0)?;
        let hierarchy: Vec<HierarchyRelation> = rows.into_iter().map(|r| HierarchyRelation {
            parent_db_id: r.parent.to_string(),
            child_db_id: r.child.to_string(),
            order: r.order,
        }).collect();

        Ok(SceneSnapshot {
            scene_id: scene_thing.id.to_string(),
            scene,
            entities,
            components,
            hierarchy,
        })
    }

    pub async fn list_scenes(&self) -> Result<Vec<SceneRecord>, DbError> {
        self.conn.use_database("scenes").await?;

        let mut result = self.conn.inner()
            .query("SELECT * FROM scene ORDER BY updated_at DESC")
            .await?;
        let scenes: Vec<SceneRecord> = result.take(0)?;
        Ok(scenes)
    }

    pub async fn find_entities(
        &self,
        scene_name: &str,
        filter: &EntityFilter,
    ) -> Result<Vec<EntityWithComponents>, DbError> {
        self.conn.use_database("scenes").await?;

        let mut surql = String::from(
            "SELECT *, (SELECT * FROM component WHERE entity = $parent.id) AS components FROM entity WHERE scene.name = $scene_name"
        );

        if filter.name.is_some() {
            surql.push_str(" AND name CONTAINS $filter_name");
        }

        if filter.tags.is_some() {
             surql.push_str(" AND tags CONTAINSANY $filter_tags");
        }

        if filter.has_component.is_some() {
             surql.push_str(" AND id IN (SELECT entity FROM component WHERE component_type = $filter_component)");
        }

        surql.push_str(" ORDER BY order");

        let mut query = self.conn.inner().query(&surql).bind(("scene_name", scene_name.to_string()));

        if let Some(name) = &filter.name {
            query = query.bind(("filter_name", name.clone()));
        }

        if let Some(tags) = &filter.tags {
             query = query.bind(("filter_tags", tags.clone()));
        }

        if let Some(component_type) = &filter.has_component {
             query = query.bind(("filter_component", component_type.clone()));
        }

        let mut result = query.await?.check()?;

        let entities: Vec<EntityWithComponents> = result.take(0)?;
        Ok(entities)
    }

    pub async fn delete_scene(&self, scene_name: &str) -> Result<(), DbError> {
        self.conn.use_database("scenes").await?;

        let response = self.conn.inner()
            .query(r#"
                BEGIN TRANSACTION;
                LET $scene = (SELECT id FROM scene WHERE name = $name);
                DELETE component WHERE entity.scene = $scene;
                DELETE parent_of WHERE in.scene = $scene;
                DELETE entity WHERE scene = $scene;
                DELETE $scene;
                COMMIT TRANSACTION;
            "#)
            .bind(("name", scene_name.to_string()))
            .await?;

        response.check()?;

        Ok(())
    }
}
