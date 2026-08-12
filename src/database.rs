use std::path::Path;
use rusqlite::Connection;

pub fn connect(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    Connection::open(db_path)
}

pub fn initialize_files_table(db: &Connection) -> rusqlite::Result<usize> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )",
        [],
    )
}

pub fn insert_row(db: &Connection, id: &str, name: &str) -> rusqlite::Result<usize> {
    db.execute(
        "INSERT INTO files (id, name) VALUES (?1, ?2)",
        (id, name),
    )
}

pub fn query_name(db: &Connection, id: &str) -> rusqlite::Result<String> {
    db.query_row(
        "SELECT name FROM files WHERE id = ?1",
        [id],
        |row| row.get::<_, String>(0),
    )
}