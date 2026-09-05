use crate::database::FileEntry;
use axum::{
    extract::{Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::Response,
    routing::get,
    Json, Router,
};
use axum::body::Body;
use futures_util::StreamExt;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use crate::{database, id, APP_DATA_DIR};

// TODO consistency pass on startup
// TODO properly encoded filenames
// TODO tracing

const MAX_UPLOAD_SIZE: i64 = 10 * 1024 * 1024 * 1024;

pub fn router(db_path: PathBuf) -> Router {
    Router::new()
        .route("/files", get(get_all_file_info).post(store_file))
        .route("/files/{id}", get(get_file).delete(remove_file))
        .with_state(db_path)
}

async fn store_file(
    State(db_path): State<PathBuf>,
    headers: HeaderMap,
    body: Body,
) -> Result<String, StatusCode> {
    let value = headers
        .get("x-file-name")
        .ok_or(StatusCode::BAD_REQUEST)?;
    let name = value
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let id = id::gen_rand_id();
    let tmp_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(format!("tmp-{id}"));
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut received = 0i64;
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        // chunk error cases are more complex, but it is a reasonable solution
        let chunk = chunk
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        received += chunk.len() as i64;

        if received > MAX_UPLOAD_SIZE {
            // worth logging at least
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }

        file.write_all(&chunk)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // rename before database insertion
    let final_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(&id);
    tokio::fs::rename(&tmp_path, &final_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // index decides what exists so it comes last
    let db = database::connect(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    database::insert_file_entry(&db, &id, name, received)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(id)
}

async fn get_file(
    State(db_path): State<PathBuf>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let db = database::connect(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let name = database::query_file_name(&db, &id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let file = tokio::fs::File::open(PathBuf::from(APP_DATA_DIR).join("files").join(&id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let mut response = Response::new(body);

    // passing the filename in two ways
    // 1. header for easy parsing in the cli client
    // 2. content disposition for downloading in the browser
    response.headers_mut().insert(
        "x-file-name",
        HeaderValue::from_str(&name)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    let disposition = format!("attachment; filename=\"{}\"", name);
    response.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::try_from(disposition)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    Ok(response)
}

// Messy logging that I should replace with tracing
async fn get_all_file_info(
    State(db_path): State<PathBuf>,
) -> Result<Json<Vec<FileEntry>>, StatusCode> {
    let db = database::connect(&db_path)
        .map_err(|error| {
            eprintln!("database connect failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let files = database::query_all_file_entries(&db)
        .map_err(|error| {
            eprintln!("query_all_file_entries failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(files))
}

async fn remove_file(
    State(db_path): State<PathBuf>,
    Path(id): Path<String>,
) -> Result<String, StatusCode> {
    let db = database::connect(&db_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // do this first because an orphan is easier to liquidate than missing entry causing runtime issues
    let name = database::delete_file_entry(&db, &id)
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    // who cares if this fails just run a cleanup job sometimes
    let file_path = PathBuf::from(APP_DATA_DIR)
        .join("files")
        .join(&id);

    tokio::fs::remove_file(file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(name)
}