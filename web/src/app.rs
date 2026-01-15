use std::{
    env, fs::File, io::Write, path::PathBuf, sync::{Arc, Mutex}
};

use axum::{
    Router,
    extract::{DefaultBodyLimit, Request},
    middleware::{self, Next},
    response::Response,
};
use axum_login::{
    AuthManagerLayerBuilder,
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
};
use axum_messages::MessagesManagerLayer;
use db::{
    db_context::{self, DbContext}, group_db, model::User, user_db
};
use service::{AppState, Configuration, StoredConfiguration, import_state::ImportState, stored_to_configuration, thumbnail_service};
use time::{Duration, OffsetDateTime};
use tokio::{fs, signal, task::AbortHandle};
use tower_http::{compression::CompressionLayer, services::{ServeDir, ServeFile}};
use tower_sessions::cookie::Key;
use tower_sessions_sqlx_store::SqliteStore;

use crate::{
    controller::{
        auth_controller, blob_controller, group_controller, label_controller, model_controller, page_controller, resource_controller, share_controller, threemf_controller, user_controller
    },
    user::{AuthSession, Backend},
    web_app_state::WebAppState, web_import_state::WebImportStateEmitter,
};

pub struct App {
    app_state: WebAppState,
    session_store: SqliteStore,
}

fn expected_env_error_msg(var_name: &str) -> String {
    format!("Expected environment variable {} to be set", var_name)
}

async fn update_session_middleware(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> Response {
    if auth_session.user.is_some() {
        let expiry_date = auth_session.session.expiry_date();
        let now = OffsetDateTime::now_utc();
        let difference = expiry_date - now;

        if difference < Duration::days(5) {
            auth_session
                .session
                .set_expiry(Some(Expiry::OnInactivity(Duration::days(7))));
        }
    }

    next.run(request).await
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let port = env::var("SERVER_PORT")
            .unwrap_or("3000".into())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid u16");

        let config_path =
            env::var("APP_CONFIG_PATH").expect(&expected_env_error_msg("APP_CONFIG_PATH"));
        let config_path = PathBuf::from(config_path);

        if !config_path.exists() {
            File::create(&config_path)?.write_all(
                serde_json::to_string_pretty(&Configuration::default())
                    .unwrap()
                    .as_bytes(),
            )?;
        }

        let json = fs::read_to_string(&config_path)
            .await
            .expect("Failed to read configuration");
        let configuration: StoredConfiguration =
            serde_json::from_str(&json).expect("Failed to parse configuration");
        let mut configuration = stored_to_configuration(configuration);

        if configuration.data_path.is_empty() {
            let default_data_dir = config_path.parent().unwrap();

            configuration.data_path = default_data_dir.to_str().unwrap().to_string();
        }

        let data_dir = PathBuf::from(configuration.data_path.clone());
        let sqlite_path = PathBuf::from(&data_dir).join("db.sqlite");
        let sqlite_backup_dir = PathBuf::from(&data_dir).join("backups");
        let db = db_context::setup_db(&sqlite_path, &sqlite_backup_dir).await;
        let db_clone = db.clone();

        let web_app_state = WebAppState {
            app_state: AppState {
                db: Arc::new(db),
                configuration: Mutex::new(configuration),
                app_data_path: data_dir.to_str().unwrap().to_string(),
                import_mutex: Arc::new(tokio::sync::Mutex::new(())),
            },
            port: port,
        };

        let session_store = SqliteStore::new(db_clone);
        session_store.migrate().await?;

        let local_pass = match env::var("LOCAL_ACCOUNT_PASSWORD") {
            Ok(password) => password,
            Err(_) => {
                let key = Key::generate();
                let key_bytes = key.master();
                key_bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<String>>()
                    .join("")
            }
        };

        println!("Password for local account for this session: {}", local_pass);

        user_db::edit_user_password(&web_app_state.app_state.db, 1, &local_pass).await?;
        user_db::scramble_validity_token(&web_app_state.app_state.db, 1).await?;
        group_db::delete_dead_groups(&web_app_state.app_state.db).await?;

        let regenerate_thumbnails = env::var("REGENERATE_THUMBNAILS").unwrap_or("none".into()).to_lowercase();

        let mut import_state = ImportState::new_with_emitter(None, false, true, false, User::default(), Box::new(WebImportStateEmitter {}));
        
        if regenerate_thumbnails == "all" {
            println!("Regenerating all thumbnails...");
            thumbnail_service::generate_all_thumbnails(&web_app_state.app_state, true, &mut import_state).await?;
        } else if regenerate_thumbnails == "missing" {
            println!("Regenerating missing thumbnails...");
            thumbnail_service::generate_all_thumbnails(&web_app_state.app_state, false, &mut import_state).await?;
        }

        Ok(Self {
            app_state: web_app_state,
            session_store,
        })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let session_store = self.session_store;

        let deletion_task = tokio::task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );

        // Generate a cryptographic key to sign the session cookie.

        let signing_key_path = self.app_state.get_signing_key_path();
        let key = match signing_key_path.exists() {
            true => {
                let key_bytes = fs::read(&signing_key_path).await?;
                Key::from(&key_bytes)
            }
            false => {
                let key = Key::generate();
                fs::write(&signing_key_path, key.master()).await?;
                key
            }
        };

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(7)))
            .with_signed(key);

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(self.app_state.app_state.db.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let serve_dir = ServeDir::new("www").not_found_service(ServeFile::new("www/index.html"));
        let db = self.app_state.app_state.db.clone();
        let port = self.app_state.port;

        let app = Router::new()
            .merge(auth_controller::router())
            .merge(blob_controller::router())
            .merge(model_controller::router())
            .merge(group_controller::router())
            .merge(label_controller::router())
            .merge(resource_controller::router())
            .merge(user_controller::router())
            .merge(threemf_controller::router())
            .merge(page_controller::router())
            .merge(share_controller::router())
            .with_state(self.app_state)
            .layer(middleware::from_fn(update_session_middleware))
            .layer(MessagesManagerLayer)
            .layer(auth_layer)
            .layer(DefaultBodyLimit::disable())
            .layer(CompressionLayer::new())
            .fallback_service(serve_dir);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();

        println!("Server running on port {}", port);

        // Ensure we use a shutdown signal to abort the deletion task.
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle(), db))
            .await?;

        deletion_task.await??;

        Ok(())
    }
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle, db: Arc<DbContext>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }

    db.close().await;
}
