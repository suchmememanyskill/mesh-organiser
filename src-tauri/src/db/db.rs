use sqlx;
use tauri::State;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, prelude::FromRow, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs::OpenOptions;
use tauri::{App, Manager as _};

pub type Db = Pool<Sqlite>;

pub async fn setup_db(app: &App) -> Db {
    let mut path = app.path().app_data_dir().expect("failed to get data_dir");
 
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
 
    if !Sqlite::database_exists(url.as_str()).await.unwrap()
    {
        Sqlite::create_database(url.as_str())
        .await
        .expect("failed to create database");
    };
 
    let db = SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .unwrap();
 
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
 
    db
}