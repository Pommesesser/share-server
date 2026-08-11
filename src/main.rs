mod storage;

use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;

const DATA_DIR_STR: &str = "./data";

fn main() {
    initialize_app_data_dir();
    initialize_index();
}

fn initialize_app_data_dir() {
    let app_data_dir = PathBuf::from(DATA_DIR_STR);
    fs::create_dir_all(app_data_dir.join("files"))
        .expect("Unable to initialize app data directory");
}

fn initialize_index() {
    let index_path = PathBuf::from(DATA_DIR_STR).join("index.db");

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
}