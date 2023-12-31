use anyhow::Result;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use zero2prod::configuration::Settings;
use zero2prod::startup::run;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info", std::io::stdout);
    telemetry::init_subscriber(subscriber)?;

    let configuration = Settings::get_configuration()?;
    let pg_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret()).await?;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(address)?;

    run(listener, pg_pool)?.await?;

    Ok(())
}
