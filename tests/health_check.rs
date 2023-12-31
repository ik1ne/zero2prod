use anyhow::Result;
use sqlx::query;

use common::spawn_app;

use crate::common::TestApp;

mod common;

#[tokio::test]
async fn health_check_works() -> Result<()> {
    let TestApp {
        address,
        db_pool: _,
    } = spawn_app().await?;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<()> {
    let TestApp { address, db_pool } = spawn_app().await?;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 200);

    let saved = query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&db_pool)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> Result<()> {
    let TestApp {
        address,
        db_pool: _,
    } = spawn_app().await?;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", address))
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
