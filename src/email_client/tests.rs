use anyhow::Result;
use fake::faker::internet::en;
use fake::faker::lorem;
use fake::{Fake, Faker};
use secrecy::Secret;
use serde_json::Value;
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

use crate::email_client::EmailClient;

#[tokio::test]
async fn send_email_sends_the_expected_request() -> Result<()> {
    let mock_server = MockServer::start().await;
    let sender: String = en::SafeEmail().fake();

    let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

    Mock::given(header_exists("X-Postmark-Server-Token"))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let subscriber_email: String = en::SafeEmail().fake();
    let subject: String = lorem::en::Sentence(1..2).fake();
    let content: String = lorem::en::Paragraph(1..10).fake();

    email_client
        .send_email(subscriber_email, &subject, &content, &content)
        .await?;

    Ok(())
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
