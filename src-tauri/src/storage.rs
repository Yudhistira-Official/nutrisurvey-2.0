use crate::error::AppError;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Executor, Sqlite, SqlitePool, Transaction,
};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub struct Storage {
    pool: SqlitePool,
    app_data_dir: PathBuf,
    resource_dir: PathBuf,
}

impl Storage {
    pub async fn open(app_handle: &AppHandle) -> Result<Self, AppError> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Io(e.to_string()))?;
        let resource_dir = app_handle
            .path()
            .resource_dir()
            .map_err(|e| AppError::Io(e.to_string()))?;
        Self::open_at(app_data_dir, resource_dir, None).await
    }

    pub async fn open_path(path: &Path) -> Result<Self, AppError> {
        let resource_dir = PathBuf::new();
        Self::open_paths(path, &resource_dir).await
    }

    pub async fn open_paths(path: &Path, resource_dir: &Path) -> Result<Self, AppError> {
        let app_data_dir = path
            .parent()
            .ok_or_else(|| AppError::Io("database path has no parent".into()))?
            .to_path_buf();
        Self::open_at(
            app_data_dir,
            resource_dir.to_path_buf(),
            Some(path.to_path_buf()),
        )
        .await
    }

    async fn open_at(
        app_data_dir: PathBuf,
        resource_dir: PathBuf,
        database_path: Option<PathBuf>,
    ) -> Result<Self, AppError> {
        tokio::fs::create_dir_all(&app_data_dir).await?;
        let database_path =
            database_path.unwrap_or_else(|| app_data_dir.join("nutrisurvey.sqlite3"));
        let options = SqliteConnectOptions::new()
            .filename(&database_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .after_connect(|connection, _| {
                Box::pin(async move {
                    connection.execute("PRAGMA foreign_keys = ON").await?;
                    Ok(())
                })
            })
            .connect_with(options)
            .await?;
        let storage = Self {
            pool,
            app_data_dir,
            resource_dir,
        };
        storage.initialize().await?;
        Ok(storage)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }

    pub fn resource_dir(&self) -> &Path {
        &self.resource_dir
    }

    pub async fn initialize(&self) -> Result<(), AppError> {
        sqlx::raw_sql(include_str!("../migrations/0001_initial.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn transaction(&self) -> Result<Transaction<'_, Sqlite>, AppError> {
        Ok(self.pool.begin().await?)
    }
}
