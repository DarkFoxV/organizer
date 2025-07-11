use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn get_connection() -> Result<DatabaseConnection, DbErr> {
    let db_url = "sqlite://organizer.db?mode=rwc";
    Database::connect(db_url).await
}
