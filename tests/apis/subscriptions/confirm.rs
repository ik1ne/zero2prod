use crate::common::TestApp;

use crate::subscriptions::get_single_link;
use anyhow::{Context, Result};
use reqwest::Url;
use wiremock::matchers::path;
use wiremock::Mock;

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
    let email_request = test_app
        .email_server
        .received_requests()
        .await
        .context("No requests")?
        .pop()
        .context("Empty requests")?;

    let body: serde_json::Value =
        serde_json::from_slice(&email_request.body).context("Invalid body")?;
    let raw_confirmation_link = get_single_link(body["HtmlBody"].as_str().context("No htmlBody")?)?;
    let confirmation_link = Url::parse(&raw_confirmation_link)?;

    assert_eq!(confirmation_link.host_str(), Some("localhost"));

    let response = reqwest::Client::new()
        .post(confirmation_link)
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 200);

    Ok(())
}
