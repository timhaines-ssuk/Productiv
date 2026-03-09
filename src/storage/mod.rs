mod activity;
mod schedule;
mod schema;
mod settings;
mod shared;
mod tasks;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow};
use rusqlite::Connection;

#[derive(Clone)]
pub struct Database {
    path: Arc<PathBuf>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let local_app_data =
            dirs::data_local_dir().ok_or_else(|| anyhow!("LocalAppData directory not found"))?;
        let app_dir = local_app_data.join("Productiv");
        fs::create_dir_all(&app_dir).context("failed to create Productiv data directory")?;
        let path = app_dir.join("productiv.sqlite3");
        let database = Self {
            path: Arc::new(path),
        };
        database.initialize()?;
        Ok(database)
    }

    pub fn db_path(&self) -> &Path {
        self.path.as_ref().as_path()
    }

    fn connection(&self) -> Result<Connection> {
        let connection = Connection::open(self.path.as_ref())
            .with_context(|| format!("failed to open {}", self.path.display()))?;
        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            ",
        )?;
        Ok(connection)
    }

    fn initialize(&self) -> Result<()> {
        let connection = self.connection()?;
        schema::initialize_schema(&connection)
    }
}
