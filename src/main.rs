use anyhow::Result;
use sqlx::PgPool;

use zero2prod::configuration::Settings;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = Settings::get_configuration()?;
    let pg_pool = PgPool::connect(&configuration.database.connection_string()).await?;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;

    run(listener, pg_pool)?.await?;

    Ok(())
}
