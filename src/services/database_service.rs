use log::{error, info};
use migration::Migrator;
use sea_orm_migration::MigratorTrait;
use std::{error::Error, fs, path::Path, time::Instant};
use std::path::PathBuf;
use crate::services::connection_db::{db_ref};
use crate::utils::get_exe_dir;

pub async fn run_migrations_safe(db: &sea_orm::DatabaseConnection) -> Result<(), Box<dyn Error>> {
    info!("Iniciando verificação de migrações...");
    let start = Instant::now();

    // Verifica se o banco está acessível
    db.ping().await.map(|_| info!("Conexão com banco OK"))?;

    // Pegar migrações pendentes
    let pending = Migrator::get_pending_migrations(db).await.map_err(|e| {
        error!("Erro ao verificar migrações: {}", e);
        e
    })?;

    if pending.is_empty() {
        info!("Schema do banco está atualizado.");
        return Ok(());
    }

    info!("Há {} migração(ões) pendente(s):", pending.len());
    for m in &pending {
        info!("  - {}", m.name());
    }

    // Backup antes de aplicar migrações
    backup_database().await.map_err(|e| {
        error!("Falha ao criar backup antes das migrations: {}", e);
        e
    })?;

    // Aplicar migrações
    Migrator::up(db, None)
        .await
        .map(|_| {
            let duration = start.elapsed();
            info!("Migrações aplicadas com sucesso em {:?}", duration);
        })
        .map_err(|e| {
            error!("Erro ao aplicar migrações: {}", e);
            e
        })?;

    Ok(())
}

pub async fn prepare_database() -> Result<(), Box<dyn Error>> {
    let db_path = "organizer.db";
    let is_fresh = !Path::new(db_path).exists();

    // Cria uma única conexão e reutiliza
    let db = db_ref();

    // Verifica se o banco responde
    db.ping().await.map_err(|e| {
        error!("Erro ao pingar banco: {}", e);
        e
    })?;

    if is_fresh {
        info!("Banco novo detectado. Aplicando todas as migrações...");
        Migrator::up(db, None).await.map_err(|e| {
            error!("Erro ao aplicar migrações no banco novo: {}", e);
            e
        })?;
        info!("Banco preparado com sucesso.");
    } else {
        info!("Banco existente. Verificando migrações pendentes...");
        run_migrations_safe(db).await?;
    }

    Ok(())
}

pub async fn backup_database() -> Result<(), Box<dyn Error>> {
    let exe_dir = get_exe_dir();
    let db_path: PathBuf = exe_dir.join("organizer.db");

    if db_path.exists() {
        let backup_path = format!(
            "database_backup_{}.db",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        fs::copy(&db_path, &backup_path)?;
        info!("Backup created: {}", backup_path);
    } else {
        info!("Database file not found at {:?}", db_path);
    }

    Ok(())
}
