use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

// This function obtains a database connection to the postgresql database used for the bot.
pub async fn obtain_postgres_pool() -> Result<PgPool, Box<dyn std::error::Error + Send + Sync>> {
    // Obtain the postgresql url.
    let pg_url = env::var("DATABASE_URL")?;

    // Connect to the database with the information provided on the configuration.
    // and return a pool of connections
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&pg_url)
        .await?;

    // return the pool
    Ok(pool)
}
