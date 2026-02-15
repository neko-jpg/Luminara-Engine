use criterion::{criterion_group, criterion_main, Criterion};
use luminara_db::prelude::*;
use tokio::runtime::Runtime;

fn db_insert_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("insert_asset", |b| {
        b.to_async(&rt).iter(|| async {
            let config = DbConfig {
                backend: DbBackend::Memory,
                auto_migrate: true,
                ..Default::default()
            };
            let conn = DbConnection::connect(config).await.unwrap();
            let store = AssetStore::new(&conn);

            let meta = AssetMeta::default();
            store.register(&meta).await.unwrap();
        })
    });
}

fn db_query_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = DbConfig {
        backend: DbBackend::Memory,
        auto_migrate: true,
        ..Default::default()
    };

    let (conn, uuid) = rt.block_on(async {
        let conn = DbConnection::connect(config).await.unwrap();
        let store = AssetStore::new(&conn);
        let meta = AssetMeta::default();
        let uuid = meta.uuid;
        store.register(&meta).await.unwrap();
        (conn, uuid)
    });

    c.bench_function("get_asset", |b| {
        b.to_async(&rt).iter(|| async {
            let store = AssetStore::new(&conn);
            store.get_by_uuid(&uuid).await.unwrap();
        })
    });
}

criterion_group!(benches, db_insert_benchmark, db_query_benchmark);
criterion_main!(benches);
