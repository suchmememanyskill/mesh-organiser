use sqlx;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

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

    let db = SqlitePoolOptions::new()
        .connect(sqlite_path.to_str().unwrap())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    backup_db(sqlite_path, sqlite_backup_dir);

    db
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
