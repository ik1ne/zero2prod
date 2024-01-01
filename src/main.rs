use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

use zero2prod::configuration::Settings;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info", std::io::stdout);
    telemetry::init_subscriber(subscriber)?;

    let configuration = Settings::get_configuration()?;

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = std::net::TcpListener::bind(address)?;

    let pg_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender_email,
        configuration.email_client.authorization_token,
    );

    run(listener, pg_pool, email_client)?.await?;

    Ok(())
}
