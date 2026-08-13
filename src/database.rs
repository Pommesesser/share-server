use std::path::Path;
use rusqlite::Connection;

#[derive(serde::Serialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
}

pub fn connect(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    Connection::open(db_path)
}

pub fn initialize_files_table(db: &Connection) -> rusqlite::Result<usize> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            data BLOB NOT NULL
        )",
        [],
    )
}

pub fn insert_file(db: &Connection, id: &str, name: &str, data: &[u8]) -> rusqlite::Result<usize> {
    db.execute(
        "INSERT INTO files (id, name, data) VALUES (?1, ?2, ?3)",
        (id, name, data),
    )
}

pub fn query_all_file_info(db: &Connection)-> rusqlite::Result<Vec<FileInfo>> {
    let mut statement = db.prepare(
        "SELECT id, name FROM files"
    )?;

    let rows = statement.query_map([], |row| {
        Ok(FileInfo {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    rows.collect()
}

pub fn delete_file(db: &Connection, id: &str) -> rusqlite::Result<usize> {
    db.execute(
        "DELETE FROM files WHERE id = ?1",
        [id],
    )
}

pub fn query_file(db: &Connection, id: &str) -> rusqlite::Result<(String, Vec<u8>)> {
    db.query_row(
        "SELECT name, data FROM files WHERE id = ?1",
        [id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Vec<u8>>(1)?,
            ))
        },
    )
}