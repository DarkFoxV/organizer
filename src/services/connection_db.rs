use crate::utils::get_exe_dir;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::{sync::Arc, time::Duration};

static DB: OnceCell<Arc<DatabaseConnection>> = OnceCell::new();

pub async fn init_db() -> Result<(), DbErr> {
    let exe_dir = get_exe_dir();
    let db_path = exe_dir.join("organizer.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5)
        .connect_timeout(Duration::from_secs(3))
        .sqlx_logging(false);

    let db = Database::connect(opt).await?;

    DB.set(Arc::new(db))
        .map_err(|_| DbErr::Custom("DB already initialized".into()))?;

    Ok(())
}

pub fn db_ref() -> &'static DatabaseConnection {
    DB.get()
        .expect("DB not initialized. Call init_db() first")
        .as_ref()
}