use sea_orm::{Database, DatabaseConnection, DbErr};
use crate::utils::get_exe_dir;

pub async fn get_connection() -> Result<DatabaseConnection, DbErr> {
    // Get the directory of the executable
    let exe_dir = get_exe_dir();

    // Path to the SQLite database
    let db_path = exe_dir.join("organizer.db");

    // Build the connection string
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    Database::connect(&db_url).await
}
