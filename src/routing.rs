use crate::database::FileEntry;
use crate::storage::{OpenFileError, RemoveError, StoreFileError};
use crate::storage;
use axum::body::Body;
use axum::{
    Json, Router,
    extract::Path,
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::Response,
    routing::get,
};
use tokio_util::io::ReaderStream;

// TODO length header
// TODO authentication
// TODO consistency pass on startup
// TODO properly encoded filenames
// TODO tracing

pub const MAX_FILE_SIZE: i64 = 10 * 1024 * 1024 * 1024;

pub fn router() -> Router {
    Router::new()
        .route("/files", get(get_file_entries).post(upload_file))
        .route("/files/{id}", get(get_file).delete(delete_file))
}

async fn upload_file(headers: HeaderMap, body: Body) -> Result<String, StatusCode> {
    let value = headers
        .get("x-file-name")
        .ok_or(StatusCode::BAD_REQUEST)?;
    let name = value
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    storage::store_file(name, body.into_data_stream())
        .await.map_err(|error| match error {
        StoreFileError::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        StoreFileError::Connection |
        StoreFileError::Stream |
        StoreFileError::Database |
        StoreFileError::Filesystem => StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn get_file(Path(id): Path<String>) -> Result<Response, StatusCode> {
    let stored = storage::open_file(&id)
        .await
        .map_err(|error| match error {
            OpenFileError::Connection
            | OpenFileError::Database
            | OpenFileError::Filesystem => {
                StatusCode::INTERNAL_SERVER_ERROR
            }

            OpenFileError::NotFound => StatusCode::NOT_FOUND,
        })?;

    let mut response = Response::new(Body::from_stream(ReaderStream::new(stored.file)));

    // passing the filename in two ways
    // 1. header for easy parsing in the cli client
    // 2. content disposition for downloading in the browser
    response.headers_mut().insert(
        "x-file-name",
        HeaderValue::from_str(&stored.name)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    let disposition = format!("attachment; filename=\"{}\"", &stored.name);
    response.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::try_from(disposition)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    Ok(response)
}

async fn get_file_entries() -> Result<Json<Vec<FileEntry>>, StatusCode> {
    storage::get_file_entries()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn delete_file(Path(id): Path<String>) -> Result<String, StatusCode> {
    storage::remove(&id)
        .await
        .map_err(|error| match error {
            RemoveError::NotFound => StatusCode::NOT_FOUND,
            RemoveError::Connection
            | RemoveError::Database
            | RemoveError::Filesystem => StatusCode::INTERNAL_SERVER_ERROR,
        })
}