use anyhow::{Context, Result};
use wiremock::matchers::{method, path};
use wiremock::Mock;

use crate::common::{ConfirmationLinks, TestApp};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_404() -> Result<()> {
    let app = TestApp::new().await?;

    let response = reqwest::Client::new()
        .post(&format!("{}/subscriptions/confirm", app.address))
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 404);

    Ok(())
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() -> Result<()> {
    let test_app = TestApp::new().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .respond_with(wiremock::ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.to_string()).await?;

    let requests = test_app
        .email_server
        .received_requests()
        .await
        .context("No requests")?;
    let email_request = requests.first().context("Empty requests")?;

    let confirmation_link = ConfirmationLinks::try_from(email_request, test_app.port)?;

    let response = reqwest::Client::new()
        .post(dbg!(confirmation_link.html))
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 200);

    Ok(())
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() -> Result<()> {
    let test_app = TestApp::new().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(wiremock::ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.to_string()).await?;
    let request = test_app
        .email_server
        .received_requests()
        .await
        .context("No requests")?;
    let confirmation_link =
        ConfirmationLinks::try_from(request.first().context("Empty requests")?, test_app.port)?;

    reqwest::Client::new()
        .post(confirmation_link.html)
        .send()
        .await?;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");

    Ok(())
}
