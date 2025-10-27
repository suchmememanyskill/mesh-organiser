use sqlx;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

use crate::configuration::Configuration;

pub type Db = Pool<Sqlite>;

pub async fn setup_db(configuration: &Configuration, backup_db_path: &str) -> Db {
    let mut path = PathBuf::from(configuration.data_path.clone());

    match std::fs::create_dir_all(path.clone()) {
        Ok(_) => {}
        Err(err) => {
            panic!("error creating directory {}", err);
        }
    };

    path.push("db.sqlite");

    let url = format!(
        "sqlite:{}",
        path.to_str().expect("path should be something")
    );

    if !Sqlite::database_exists(url.as_str()).await.unwrap() {
        Sqlite::create_database(url.as_str())
            .await
            .expect("failed to create database");
    };

    let db = SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    backup_db(configuration, backup_db_path);

    db
}

fn backup_db(configuration: &Configuration, data_path: &str) {
    let mut db_path = PathBuf::from(configuration.data_path.clone());
    db_path.push("db.sqlite");
    let mut backup_dir = PathBuf::from(data_path);
    let timestamp = chrono::Utc::now().timestamp_millis();

    if !db_path.exists() {
        return;
    }

    backup_dir.push("backup");

    if !backup_dir.exists() {
        std::fs::create_dir_all(backup_dir.clone()).expect("Failed to create backup directory");
    }

    let backup_file_path = backup_dir.join(format!("{}.sqlite", timestamp));
    std::fs::copy(&db_path, &backup_file_path).expect("Failed to create backup");

    let mut backups: Vec<_> = std::fs::read_dir(&backup_dir)
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
