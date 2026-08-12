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
    Filesystem,
    Database
}

pub fn store_file(db: &Connection, file: &File) -> Result<String, StoreFileError> {
    let id = Uuid::now_v7().to_string();

    let temp_file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(format!("tmp-{}", id));

    fs::write(&temp_file_path, &file.data)
        .map_err(|_| {
            // I'm not checking because I don't care if temporary files pile up lol
            let _ = fs::remove_file(&temp_file_path);
            StoreFileError::Filesystem
        })?;

    let final_file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(&id);

    fs::rename(&temp_file_path, &final_file_path)
        .map_err(|_| {
            // Still not checking
            let _ = fs::remove_file(&temp_file_path);
            StoreFileError::Filesystem
        })?;

    // Crashing before insertion completes produces orphaned file
    database::insert_file(db, &id, &file.name)
        .map_err(|_| {
            // Produces orphaned files if this fails
            let _ = fs::remove_file(&final_file_path);
            StoreFileError::Database
        })?;

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

#[derive(Debug)]
pub enum RemoveFileError {
    FileNotFound,
    EntryNotFound,
    Filesystem,
    Database
}

pub fn remove_file(db: &Connection, id: &str) -> Result<(), RemoveFileError> {
    let file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(id);
    if !file_path.exists() {
        return Err(RemoveFileError::FileNotFound);
    }

    let entry_exists = database::file_exists(db, id)
        .map_err(|_| RemoveFileError::Database)?;

    if !entry_exists {
        return Err(RemoveFileError::EntryNotFound);
    }

    let del_file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(format!("del-{}", id));

    fs::rename(&file_path, &del_file_path)
        .map_err(|_| RemoveFileError::Filesystem)?;

    database::delete_file(db, id)
        .map_err(|_| {
            let _ = fs::rename(&del_file_path, &file_path);
            RemoveFileError::Database
        })?;

    let _ = fs::remove_file(&del_file_path);

    Ok(())
}