use std::net::TcpListener;
use std::sync::OnceLock;

use anyhow::Result;
use sqlx::{Connection, Error, Executor, PgConnection, PgPool, Pool, Postgres};

use zero2prod::configuration::{DatabaseSettings, Settings};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: OnceLock<()> = OnceLock::new();

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> Result<TestApp> {
    TRACING.get_or_init(|| {
        let subscriber_name = "test".into();
        let default_filter_level = "info";

        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber).unwrap();
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber).unwrap();
        };
    });

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = Settings::get_configuration()?;
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let pg_pool = configure_database(&mut configuration.database).await?;

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender_email,
        configuration.email_client.authorization_token,
    );

    let server = run(listener, pg_pool.clone(), email_client)?;
    drop(tokio::spawn(server));

    Ok(TestApp {
        address,
        db_pool: pg_pool,
    })
}

async fn configure_database(db_settings: &mut DatabaseSettings) -> Result<Pool<Postgres>, Error> {
    let mut connection = PgConnection::connect_with(&db_settings.without_db()).await?;
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_settings.database_name).as_str())
        .await?;

    let connection_pool = PgPool::connect_with(db_settings.with_db()).await?;

    sqlx::migrate!("./migrations").run(&connection_pool).await?;

    Ok(connection_pool)
}
