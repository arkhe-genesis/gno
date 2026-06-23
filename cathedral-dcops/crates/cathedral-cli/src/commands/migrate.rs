use sqlx::{SqlitePool, PgPool};

#[derive(clap::Subcommand)]
pub enum MigrateCommands {
    Sqlite {
        #[arg(long, default_value = "cathedral.db")]
        database: String,
    },
    Postgres {
        #[arg(long, )]
        url: String,
    },
}

pub async fn run_migrate(cmd: MigrateCommands) -> Result<(), String> {
    match cmd {
        MigrateCommands::Sqlite { database } => {
            sqlx::migrate!("../../migrations").run(&SqlitePool::connect(&database).await.map_err(|e| e.to_string())?).await.map_err(|e| e.to_string())?;
            println!("✅ Migrações SQLite aplicadas.");
        }
        MigrateCommands::Postgres { url } => {
            sqlx::migrate!("../../migrations").run(&PgPool::connect(&url).await.map_err(|e| e.to_string())?).await.map_err(|e| e.to_string())?;
            println!("✅ Migrações PostgreSQL aplicadas.");
        }
    }
    Ok(())
}
