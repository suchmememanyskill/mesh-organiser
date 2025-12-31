use sqlx;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::path::PathBuf;
use std::time::Duration;

pub type DbContext = Pool<Sqlite>;

pub async fn setup_db(sqlite_path : &PathBuf, sqlite_backup_dir : &PathBuf) -> DbContext {
    let url = format!(
        "sqlite:{}",
        sqlite_path.to_str().expect("path should be something")
    );

    if !Sqlite::database_exists(url.as_str()).await.unwrap() {
        Sqlite::create_database(url.as_str())
            .await
            .expect("failed to create database");
    };

    let connection_option = SqliteConnectOptions::new()
        .filename(sqlite_path)
        .busy_timeout(Duration::from_secs(15));

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_option)
        .await
        .unwrap();

    let migration_count = get_db_migration_count(&db).await;

    hack_fix_duplicate_model_entries(&db, migration_count).await;

    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    backup_db(sqlite_path, sqlite_backup_dir);

    let new_migration_count = get_db_migration_count(&db).await;

    if new_migration_count > migration_count {
        sqlx::query!("VACUUM")
            .execute(&db)
            .await
            .expect("Failed to vacuum database after migrations");
    }

    db
}

async fn hack_fix_duplicate_model_entries(db: &DbContext, migration_count : usize) {
    // This is only an issue between migration 1 and 5
    // See https://github.com/suchmememanyskill/mesh-organiser/issues/38 for more information
    if migration_count <= 0 || migration_count > 5 {
        return;
    }

    let _ = sqlx::query("DELETE FROM models WHERE model_id in (SELECT model_id FROM models GROUP BY model_sha256 HAVING COUNT(*) > 1)")
        .execute(db)
        .await;
}

async fn get_db_migration_count(db: &DbContext) -> usize {
    let row: (i64,) = match sqlx::query_as("SELECT COUNT(*) as count FROM _sqlx_migrations")
        .fetch_one(db)
        .await {
            Ok(r) => r,
            Err(_) => return 0,
        };
        
    row.0 as usize
}

fn backup_db(sqlite_path : &PathBuf, sqlite_backup_dir : &PathBuf) {
    let timestamp = chrono::Utc::now().timestamp_millis();

    if !sqlite_path.exists() {
        return;
    }

    if !sqlite_backup_dir.exists() {
        std::fs::create_dir_all(sqlite_backup_dir).expect("Failed to create backup directory");
    }

    let backup_file_path = sqlite_backup_dir.join(format!("{}.sqlite", timestamp));
    std::fs::copy(sqlite_path, &backup_file_path).expect("Failed to create backup");

    let mut backups: Vec<_> = std::fs::read_dir(&sqlite_backup_dir)
        .expect("Failed to read backup directory")
        .filter_map(|entry| {
            entry.ok().filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "sqlite")
                    .unwrap_or(false)
            })
        })
        .collect();

    backups.sort_by_key(|entry| entry.metadata().and_then(|m| m.modified()).unwrap());
    while backups.len() > 5 {
        let oldest = backups.remove(0);
        std::fs::remove_file(oldest.path()).expect("Failed to remove old backup");
    }
}
