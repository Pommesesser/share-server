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

    let mut fs_file = fs::File::create(&file_path)
        .map_err(|_| StoreFileError::FileCreate)?;

    if fs_file.write_all(&file.data).is_err() {
        let _ = fs::remove_file(file_path);
        return Err(StoreFileError::FileWrite)
    }

    if database::insert_file(db, &id, &file.name).is_err() {
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

    let name = database::query_name(db, id)
        .map_err(|_| GetFileError::Database)?;

    let data = fs::read(&file_path)
        .map_err(|_| GetFileError::FileRead)?;

    Ok(File { name, data })
}

#[derive(Debug)]
pub enum GetAllFileNamesError {
    Database
}

pub fn get_all_file_names(db: &Connection) -> Result<Vec<String>, GetAllFileNamesError> {
    database::query_all_names(db)
        .map_err(|_| GetAllFileNamesError::Database)
}

pub enum RemoveFileError {
    FileNotFound
}

pub fn remove_file(db: &Connection, id: &str) -> Result<(), RemoveFileError> {
    let file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(id);
    if !file_path.exists() {
        return Err(RemoveFileError::FileNotFound);
    }

    todo!()
}