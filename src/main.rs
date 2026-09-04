mod database;
mod routing;
mod id;

use std::fs;
use std::path::{Path, PathBuf};

const APP_DATA_DIR: &str = "./data";

#[tokio::main]
async fn main() {
    let db_path = PathBuf::from(APP_DATA_DIR).join("share-server.db");

    initialize_app_data_dir();
    initialize_database(&db_path);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind server");

    axum::serve(listener, routing::router(db_path))
        .await
        .expect("Server failed");
}

fn initialize_app_data_dir() {
    fs::create_dir_all(PathBuf::from(APP_DATA_DIR).join("files"))
        .expect("Unable to initialize app data directory");
}

fn initialize_database(db_path: &Path) {
    let db = database::connect(&db_path)
        .expect("Unable to initialize database connection");

    database::initialize_files_table(&db)
        .expect("Unable to initialize files table");
}