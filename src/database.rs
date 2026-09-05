use std::path::Path;
use rusqlite::Connection;

#[derive(serde::Serialize)]
pub struct FileEntry {
    pub id: String,
    pub name: String,
    pub size: i64
}

pub fn connect(db_path: &Path) -> rusqlite::Result<Connection> {
    Connection::open(db_path)
}

pub fn initialize_files_table(db: &Connection) -> rusqlite::Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            size INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}

pub fn insert_file_entry(
    db: &Connection,
    id: &str,
    name: &str,
    size: i64,
) -> rusqlite::Result<()> {
    db.execute(
        "INSERT INTO files (id, name, size)
         VALUES (?1, ?2, ?3)",
        (id, name, size),
    )?;

    Ok(())
}

pub fn query_file_name(
    db: &Connection,
    id: &str,
) -> rusqlite::Result<String> {
    db.query_row(
        "SELECT name FROM files WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
}

pub fn query_all_file_entries(db: &Connection)-> rusqlite::Result<Vec<FileEntry>> {
    let mut statement = db.prepare(
        "SELECT id, name, size FROM files"
    )?;

    let rows = statement.query_map([], |row| {
        Ok(FileEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            size: row.get(2)?
        })
    })?;

    rows.collect()
}

pub fn delete_file_entry(
    db: &Connection,
    id: &str,
) -> rusqlite::Result<String> {
    let name = db.query_row(
        "SELECT name FROM files WHERE id = ?1",
        [id],
        |row| row.get::<_, String>(0),
    )?;

    db.execute(
        "DELETE FROM files WHERE id = ?1",
        [id],
    )?;

    Ok(name)
}