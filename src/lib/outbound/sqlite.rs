use anyhow::{anyhow, Context};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use r2d2::PooledConnection;
use std::time::Duration;
use diesel::connection::SimpleConnection;
use log::info;

mod melonbooks;
mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("resources/migrations");

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: Pool<ConnectionManager<SqliteConnection>>
}

impl Sqlite {
    pub fn new(path: &str) -> Result<Sqlite, anyhow::Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(path);
        let pool = Pool::builder()
            .max_size(1)
            .connection_timeout(Duration::from_secs(5))
            .max_lifetime(None)
            .build(manager)
            .with_context(|| format!("failed to create pool for database at {}", path))?;
        Ok(Sqlite { pool })
    }
    
    #[cfg(test)]
    pub fn new_in_memory() -> Sqlite {
        Sqlite::new(":memory:").unwrap()
    }

    pub fn setup(&self) -> Result<(), anyhow::Error> {
        info!("setting up database");
        let mut connection = self.pool.get()?;
        connection.batch_execute("PRAGMA foreign_keys = ON;")?;
        connection.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!(e))?;
        info!("database up to date");
        Ok(())
    }
    
    fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, anyhow::Error> {
       self.pool.get().with_context(|| "cannot get db connection")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_setup() {
        let db = Sqlite::new("./data/moe-scraper.sqlite").unwrap();
        db.setup().unwrap();
    }
}