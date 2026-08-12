mod storage;
mod database;

use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
use crate::storage::File;

const APP_DATA_DIR: &str = "./data";

fn main() {
    initialize_app_data_dir();
    let db = initialize_database();

    let _ = storage::remove_file(&db, "019ff1dd-cee2-79f0-b85d-2f3fd6a20270");
    println!("{:?}", storage::get_all_file_names(&db))
}

fn initialize_app_data_dir() {
    let app_data_dir = PathBuf::from(APP_DATA_DIR);
    fs::create_dir_all(app_data_dir.join("files"))
        .expect("Unable to initialize app data directory");
}

fn initialize_database() -> Connection {
    let db_path = PathBuf::from(APP_DATA_DIR).join("index.db");

    let db = database::connect(&db_path)
        .expect("Unable to initialize database connection");

    database::initialize_files_table(&db)
        .expect("Unable to initialize files table");

    db
}