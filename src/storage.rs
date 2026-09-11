use axum::body::BodyDataStream;
use futures_util::StreamExt;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::database::FileEntry;
use crate::{FILES_PATH, database, id};

pub const MAX_FILE_SIZE: i64 = 10 * 1024 * 1024 * 1024;

pub enum StoreFileError {
    Connection,
    TooLarge,
    Stream,
    Database,
    Filesystem,
}

#[tracing::instrument(skip(stream))]
pub async fn store_file(
    name: &str,
    mut stream: BodyDataStream,
) -> Result<String, StoreFileError> {
    let id = id::gen_rand_id();

    let tmp_path = PathBuf::from(FILES_PATH)
        .join(format!("tmp-{id}"));

    let mut file = File::create(&tmp_path)
        .await
        .map_err(|_| StoreFileError::Filesystem)?;

    let mut received = 0i64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|_| StoreFileError::Stream)?;

        received += chunk.len() as i64;

        if received > MAX_FILE_SIZE {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(StoreFileError::TooLarge);
        }

        file.write_all(&chunk)
            .await
            .map_err(|_| StoreFileError::Filesystem)?;
    }

    let final_path = PathBuf::from(FILES_PATH)
        .join(&id);

    tokio::fs::rename(&tmp_path, &final_path)
        .await
        .map_err(|_| StoreFileError::Filesystem)?;

    // The database is the index of files that actually exist.
    let db = database::connect()
        .map_err(|_| StoreFileError::Connection)?;

    database::insert_file_entry(&db, &id, name, received)
        .map_err(|_| StoreFileError::Database)?;

    Ok(id)
}

pub struct StoredFile {
    pub name: String,
    pub file: File,
}

pub enum OpenFileError {
    Connection,
    NotFound,
    Database,
    Filesystem,
}

#[tracing::instrument]
pub async fn open_file(id: &str) -> Result<StoredFile, OpenFileError> {
    let db = database::connect()
        .map_err(|_| OpenFileError::Connection)?;

    let name = database::query_file_name(&db, id)
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => OpenFileError::NotFound,
            _ => OpenFileError::Database,
        })?;

    let path = PathBuf::from(FILES_PATH).join(id);

    let file = File::open(path)
        .await
        .map_err(|_| OpenFileError::Filesystem)?;

    Ok(StoredFile { name, file })
}

pub enum GetFileEntriesError {
    Connection,
    Database,
}

#[tracing::instrument]
pub async fn get_file_entries() -> Result<Vec<FileEntry>, GetFileEntriesError> {
    let db = database::connect()
        .map_err(|_| {
            GetFileEntriesError::Connection
        })?;

    let files = database::query_all_file_entries(&db)
        .map_err(|_| {
            GetFileEntriesError::Database
        })?;

    Ok(files)
}

pub enum RemoveError {
    Connection,
    NotFound,
    Database,
    Filesystem,
}

#[tracing::instrument]
pub async fn remove(id: &str) -> Result<String, RemoveError> {
    let db = database::connect()
        .map_err(|_| RemoveError::Connection)?;

    let name = database::delete_file_entry(&db, id)
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => RemoveError::NotFound,
            _ => RemoveError::Database,
        })?;

    let file_path = PathBuf::from(FILES_PATH).join(id);

    tokio::fs::remove_file(file_path)
        .await
        .map_err(|_| RemoveError::Filesystem)?;

    Ok(name)
}
