mod storage;

use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
use crate::storage::File;

const APP_DATA_DIR: &str = "./data";

fn main() {
    initialize_app_data_dir();
    let db = initialize_index();

    let file = File {
        name: "duplicate".to_string(),
        data: b"hello dis is a duplicate".to_vec()
    };

    storage::store_file(&db, &file).expect("Storage failed lul");
    storage::store_file(&db, &file).expect("Storage failed lul");
}

fn initialize_app_data_dir() {
    let app_data_dir = PathBuf::from(APP_DATA_DIR);
    fs::create_dir_all(app_data_dir.join("files"))
        .expect("Unable to initialize app data directory");
}

fn initialize_index() -> Connection {
    let index_path = PathBuf::from(APP_DATA_DIR).join("index.db");

    let db = Connection::open(index_path)
        .expect("Unable to initialize index");

    db.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )",
        [],
    )
        .expect("Unable to initialize files table");

    db
}