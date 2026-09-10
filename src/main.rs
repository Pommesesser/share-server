mod database;
mod routing;
mod id;
mod storage;

use std::fs;
use std::path::PathBuf;

pub const FILES_PATH: &str = "./data/files";
pub const DB_PATH: &str = "./data/share-server.db";

#[tokio::main]
async fn main() {
    initialize_app_data_dir();
    initialize_database();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind server");

    axum::serve(listener, routing::router())
        .await
        .expect("Server failed");
}

fn initialize_app_data_dir() {
    fs::create_dir_all(PathBuf::from(FILES_PATH))
        .expect("Unable to initialize app data directory");
}

fn initialize_database() {
    let db = database::connect()
        .expect("Unable to initialize database connection");

    database::initialize_files_table(&db)
        .expect("Unable to initialize files table");
}