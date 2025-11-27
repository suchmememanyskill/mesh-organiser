#![warn(clippy::large_futures)]
#![warn(clippy::large_stack_frames)]

use std::time::Duration;

use tokio::time;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::App, error::ApplicationError};

mod app;
mod controller;
mod error;
mod user;
mod web_app_state;
mod web_thumbnail_service;

fn remove_temp_paths() -> Result<(), ApplicationError> {
    let threshold = std::time::Duration::from_secs(5 * 60);
    let now = std::time::SystemTime::now();
    for entry in std::fs::read_dir(&std::env::temp_dir())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("meshorganiser_")
        {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if now
                        .duration_since(modified)
                        .unwrap_or(std::time::Duration::ZERO)
                        >= threshold
                    {
                        println!("Removing temporary path {:?}", path);
                        std::fs::remove_dir_all(&path)?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn loop_remove_temp_paths() {
    loop {
        time::sleep(Duration::from_secs(60 * 60)).await;
        let _ = remove_temp_paths();
    }
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    tokio::spawn(loop_remove_temp_paths());

    App::new().await?.serve().await
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        //.thread_stack_size(1 * 1024 * 1024 * 1024)
        .build()
        .unwrap()
        .block_on(async {
            async_main().await.unwrap()
        })
}
