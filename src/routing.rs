use crate::database::FileInfo;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::Response,
    routing::get,
    Json, Router,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::database;

pub fn router(db: Connection) -> Router {
    let db = Arc::new(Mutex::new(db));

    Router::new()
        .route("/files", get(get_all_file_info).post(store_file))
        .route("/files/{id}", get(get_file).delete(remove_file))
        .with_state(db)
}

async fn get_all_file_info(
    State(db): State<Arc<Mutex<Connection>>>,
) -> Result<Json<Vec<FileInfo>>, StatusCode> {
    let db = db.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let files = database::query_all_file_info(&db)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(files))
}

async fn store_file(
    State(db): State<Arc<Mutex<Connection>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<String, StatusCode> {
    let name = headers
        .get("x-file-name")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let id = Uuid::now_v7().to_string();

    let db = db.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    database::insert_file(&db, &id, name, &body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(id)
}

async fn get_file(
    State(db): State<Arc<Mutex<Connection>>>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let db = db.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (name, data) = database::query_file(&db, &id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let mut response = Response::new(data.into());

    // This shit is maybe just straight up tech debt
    // I am passing the filename in two ways
    // As a header for easy parsing in the cli client
    // And in the content disposition for downloading in the browser
    response.headers_mut().insert(
        "x-file-name",
        HeaderValue::from_str(&name).expect("invalid filename"),
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

async fn remove_file(
    State(db): State<Arc<Mutex<Connection>>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let db = db.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = database::delete_file(&db, &id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
