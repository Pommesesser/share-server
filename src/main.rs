mod database;
mod routing;
mod id;

use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;

const APP_DATA_DIR: &str = "./data";

#[tokio::main]
async fn main() {
    initialize_app_data_dir();
    let db = initialize_database();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind server");

    axum::serve(listener, routing::router(db))
        .await
        .expect("Server failed");
}

fn initialize_app_data_dir() {
    let app_data_dir = PathBuf::from(APP_DATA_DIR);
    fs::create_dir_all(app_data_dir)
        .expect("Unable to initialize app data directory");
}

fn initialize_database() -> Connection {
    let db_path = PathBuf::from(APP_DATA_DIR).join("share-server.db");

    let db = database::connect(&db_path)
        .expect("Unable to initialize database connection");

    database::initialize_files_table(&db)
        .expect("Unable to initialize files table");

    db
}