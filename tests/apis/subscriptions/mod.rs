use anyhow::{bail, Context, Result};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::common::{ConfirmationLinks, TestApp};

mod confirm;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<()> {
    let test_app = TestApp::new().await?;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.to_string()).await?;

    assert_eq!(response.status().as_u16(), 200);

    Ok(())
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() -> Result<()> {
    let test_app = TestApp::new().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await?;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");

    Ok(())
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> Result<()> {
    let test_app = TestApp::new().await?;

    const TEST_CASES: [(&str, &str); 3] = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, message) in TEST_CASES {
        let response = test_app.post_subscriptions(body.to_string()).await?;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            message
        );
    }

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<()> {
    let app = TestApp::new().await?;
    const TEST_CASES: [(&str, &str); 3] = [
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in TEST_CASES {
        let response = app.post_subscriptions(body.to_string()).await?;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }

    Ok(())
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() -> Result<()> {
    let test_app = TestApp::new().await?;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.to_string()).await?;

    Ok(())
}
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() -> Result<()> {
    let test_app = TestApp::new().await?;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.to_string()).await?;

    let requests = test_app
        .email_server
        .received_requests()
        .await
        .context("No requests")?;
    let email_request = requests.first().context("Empty requests")?;

    let confirmation_links = ConfirmationLinks::try_from(email_request, test_app.port)?;

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    Ok(())
}
