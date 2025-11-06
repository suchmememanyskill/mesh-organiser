use std::path::PathBuf;

use actix_web::{App, HttpResponse, HttpServer, get, middleware, web};
use actix_cors::Cors;
use async_zip::base::read::seek::ZipFileReader;
use db::{model::User, model_db};
use tauri::{AppHandle, Manager};
use tokio::{fs::File, io::BufReader};
use tokio_util::{io::ReaderStream, compat::FuturesAsyncReadCompatExt};

use crate::{service::app_state::AppState, util::is_zipped_file_extension};

struct TauriAppState {
    app: AppHandle,
}

#[actix_web::main]
pub async fn init(app: AppHandle) -> std::io::Result<()> {
    let tauri_app = web::Data::new(TauriAppState {
        app,
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .app_data(tauri_app.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(download_model)
    })
    .bind(("127.0.0.1", 35615))?
    .run()
    .await
}

#[get("/models/{id}")]
pub async fn download_model(id: web::Path<u32>, data: web::Data<TauriAppState>) -> actix_web::HttpResponse {
    let app_state = data.app.state::<AppState>();

    let id = id.into_inner() as i64;
    let model = match model_db::get_models_via_ids(&app_state.db, &User::default(), vec![id]).await {
        Ok(m) => m,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to retrieve model"),
    };

    if model.len() <= 0 {
        return HttpResponse::NotFound().body("Model not found");
    }

    let model = &model[0];
    let base_dir = PathBuf::from(app_state.get_model_dir());
    let src_file_path = base_dir.join(format!("{}.{}", model.blob.sha256, model.blob.filetype));

    let file = match File::open(src_file_path).await {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open model file"),
    };

    let buffered_reader = BufReader::new(file);

    if is_zipped_file_extension(&model.blob.filetype) {
        let archive = match ZipFileReader::with_tokio(buffered_reader).await {
            Ok(a) => a,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to read zip archive"),
        };
        let file = match archive.into_entry(0).await {
            Ok(f) => f,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to read file from zip archive"),
        };

        let stream = ReaderStream::new(file.compat());

        return HttpResponse::Ok()
            .content_type("application/octet-stream")
            .streaming(stream);
    } 
    else {
        let stream = ReaderStream::new(buffered_reader);

        return HttpResponse::Ok()
            .content_type("application/octet-stream")
            .streaming(stream);
    }
}