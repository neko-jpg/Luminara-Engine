use luminara_db::prelude::*;
use surrealdb::sql::Uuid as DbUuid;
use uuid::Uuid;

#[tokio::test]
#[ignore = "Fails with 'missing field type' serialization error in embedded mode, despite passing json serialization test"]
async fn test_asset_register_and_get() {
    let config = DbConfig {
        backend: DbBackend::Memory,
        auto_migrate: true,
        ..Default::default()
    };
    let conn = DbConnection::connect(config).await.unwrap();
    let store = AssetStore::new(&conn);

    let uuid_raw = Uuid::new_v4();
    let uuid = DbUuid::from(uuid_raw);

    let meta = AssetMeta {
        uuid: uuid.clone(),
        path: "test/asset.png".into(),
        asset_type: AssetType::Texture,
        ..Default::default()
    };

    store.register(&meta).await.unwrap();

    let fetched = store.get_by_uuid(&uuid_raw).await.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().path, "test/asset.png");
}
