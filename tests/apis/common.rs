use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use sqlx::{Connection, Error, Executor, PgConnection, PgPool, Pool, Postgres};
use wiremock::MockServer;

use zero2prod::configuration::{DatabaseSettings, Settings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: OnceLock<()> = OnceLock::new();

pub struct TestApp {
    pub address: String,
    pub port: u16,
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
        let port = application.port();
        let address = format!("http://127.0.0.1:{}", port);

        drop(tokio::spawn(application.run_until_stopped()));

        Ok(TestApp {
            address,
            port,
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

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl ConfirmationLinks {
    pub fn try_from(value: &wiremock::Request, port: u16) -> Result<Self> {
        let body: serde_json::Value =
            serde_json::from_slice(&value.body).context("Invalid body")?;

        let html = get_single_link(body["HtmlBody"].as_str().context("No htmlBody")?, port)?;
        let plain_text = get_single_link(body["TextBody"].as_str().context("No textBody")?, port)?;

        Ok(Self { html, plain_text })
    }
}

fn get_single_link(s: &str, port: u16) -> Result<reqwest::Url> {
    let mut links = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url);

    let link = links.next().context("No links found")?.as_str().to_string();

    if links.next().is_some() {
        bail!("More than one link found");
    }

    let mut url = reqwest::Url::parse(&link)?;

    url.set_port(Some(port))
        .map_err(|_| anyhow::anyhow!("Cannot set port"))?;

    Ok(url)
}
