use crate::db;
use crate::configuration;

pub struct AppState {
    pub db: db::db::Db,
    pub configuration: configuration::Configuration
}