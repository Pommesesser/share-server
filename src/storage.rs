use std::fs;
use std::io::Write;
use std::path::PathBuf;
use rusqlite::Connection;
use uuid::Uuid;
use crate::{database, APP_DATA_DIR};

pub struct File {
    pub name: String,
    pub data: Vec<u8>
}

#[derive(Debug)]
pub enum StoreFileError {
    FileCreate,
    FileWrite,
    Database
}

pub fn store_file(db: &Connection, file: &File) -> Result<String, StoreFileError> {
    let id = Uuid::now_v7().to_string();

    let file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(&id);

    let mut fs_file = match fs::File::create(&file_path) {
        Ok(fs_file) => fs_file,
        Err(_) => return Err(StoreFileError::FileCreate)
    };

    if fs_file.write_all(&file.data).is_err() {
        let _ = fs::remove_file(file_path);
        return Err(StoreFileError::FileWrite)
    }

    if database::insert_row(db, &id, &file.name).is_err() {
        let _ = fs::remove_file(file_path);
        return Err(StoreFileError::Database)
    }

    Ok(id)
}

#[derive(Debug)]
pub enum GetFileError {
    FileNotFound,
    FileRead,
    Database
}

pub fn get_file(db: &Connection, id: &str) -> Result<File, GetFileError> {
    let file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(id);

    if !file_path.exists() {
        return Err(GetFileError::FileNotFound);
    }

    let name = match database::query_name(db, &id) {
        Ok(name) => name,
        Err(_) => return Err(GetFileError::Database),
    };

    let data = match fs::read(&file_path) {
        Ok(data) => data,
        Err(_) => return Err(GetFileError::FileRead),
    };

    Ok(File { name, data })
}
