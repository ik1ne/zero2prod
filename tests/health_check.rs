use std::net::{SocketAddr, TcpListener};

use anyhow::Result;
use sqlx::{query, Connection, PgConnection};

use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() -> Result<()> {
    let addr = spawn_app()?;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/health_check", addr))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<()> {
    let addr = spawn_app()?;
    let configuration = get_configuration()?;
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string).await?;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("http://{}/subscriptions", addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 200);

    let saved = query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> Result<()> {
    let addr = spawn_app()?;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, message) in test_cases {
        let response = client
            .post(format!("http://{}/subscriptions", addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            message
        );
    }

    Ok(())
}

pub fn spawn_app() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let server = zero2prod::startup::run(listener)?;

    drop(tokio::spawn(server));

    Ok(addr)
}
