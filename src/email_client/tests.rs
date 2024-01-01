use anyhow::Result;
use claims::assert_err;
use fake::faker::internet::en;
use fake::faker::lorem;
use fake::{Fake, Faker};
use secrecy::Secret;
use serde_json::Value;
use std::time::Duration;
use wiremock::matchers::{any, header, header_exists, method, path};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

use crate::email_client::EmailClient;

#[tokio::test]
async fn send_email_sends_the_expected_request() -> Result<()> {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(header_exists("X-Postmark-Server-Token"))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    email_client
        .send_email(&email(), &subject(), &content(), &content())
        .await?;

    Ok(())
}

#[tokio::test]
async fn send_email_succeeds_if_the_server_returns_200() -> Result<()> {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    email_client
        .send_email(&email(), &subject(), &content(), &content())
        .await?;

    Ok(())
}

#[tokio::test]
async fn send_email_fails_if_the_server_returns_500() -> Result<()> {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(any())
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    let outcome = email_client
        .send_email(&email(), &subject(), &content(), &content())
        .await;

    assert_err!(outcome);

    Ok(())
}

#[tokio::test]
async fn send_email_times_out_if_the_server_takes_too_long() {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
    Mock::given(any())
        .respond_with(response)
        .expect(1)
        .mount(&mock_server)
        .await;

    let outcome = email_client
        .send_email(&email(), &subject(), &content(), &content())
        .await;

    assert_err!(outcome);
}

struct SendEmailBodyMatcher;

impl Match for SendEmailBodyMatcher {
    fn matches(&self, request: &Request) -> bool {
        let Ok(body) = serde_json::from_slice::<Value>(&request.body) else {
            return false;
        };

        body.get("From").is_some()
            && body.get("To").is_some()
            && body.get("Subject").is_some()
            && body.get("HtmlBody").is_some()
            && body.get("TextBody").is_some()
    }
}

fn subject() -> String {
    lorem::en::Sentence(1..2).fake()
}

fn content() -> String {
    lorem::en::Paragraph(1..10).fake()
}

fn email() -> String {
    en::SafeEmail().fake()
}

fn email_client(uri: String) -> EmailClient {
    EmailClient::new(
        uri,
        email(),
        Secret::new(Faker.fake()),
        Duration::from_millis(200),
    )
}
