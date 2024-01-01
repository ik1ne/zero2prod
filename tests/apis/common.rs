use std::sync::OnceLock;

use anyhow::Result;
use sqlx::{Connection, Error, Executor, PgConnection, PgPool, Pool, Postgres};
use wiremock::MockServer;

use zero2prod::configuration::{DatabaseSettings, Settings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: OnceLock<()> = OnceLock::new();

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn new() -> Result<TestApp> {
        init_tracing();

        let mut configuration = Settings::get_configuration()?;
        configuration.database.database_name = uuid::Uuid::new_v4().to_string();
        configuration.application.port = 0;

        configure_database(&mut configuration.database).await?;

        let email_server = MockServer::start().await;
        configuration.email_client.base_url = email_server.uri();

        let application = Application::build(configuration.clone()).await?;

        let address = format!("http://127.0.0.1:{}", application.port());

        drop(tokio::spawn(application.run_until_stopped()));

        Ok(TestApp {
            address,
            db_pool: get_connection_pool(&configuration.database),
            email_server,
        })
    }

    pub async fn post_subscriptions(&self, body: String) -> Result<reqwest::Response> {
        let response = reqwest::Client::new()
            .post(format!("{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        Ok(response)
    }
}

fn init_tracing() {
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