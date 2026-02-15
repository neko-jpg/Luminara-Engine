use luminara_db::prelude::*;
use surrealdb::sql::Datetime;

#[tokio::test]
async fn test_undo_push_pop() {
    let config = DbConfig {
        backend: DbBackend::Memory,
        auto_migrate: true,
        ..Default::default()
    };
    let conn = DbConnection::connect(config).await.unwrap();
    let store = UndoStore::new(&conn, 10);

    let entry = UndoEntry {
        id: None,
        group_id: "g1".into(),
        sequence: 0,
        command_type: "test".into(),
        description: "test".into(),
        forward_data: serde_json::json!({}),
        backward_data: serde_json::json!({}),
        timestamp: Datetime::from(chrono::Utc::now()),
    };

    store.push(&entry).await.unwrap();

    let popped = store.pop_undo().await.unwrap();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap().group_id, "g1");

    let empty = store.pop_undo().await.unwrap();
    assert!(empty.is_none());
}
