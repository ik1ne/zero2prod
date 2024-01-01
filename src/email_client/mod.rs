use anyhow::Result;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;
use std::time::Duration;

#[derive(Debug)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: String,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        url: String,
        sender: String,
        authorization_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

        Self {
            http_client,
            base_url: url,
            sender,
            authorization_token,
        }
    }

    #[tracing::instrument(skip(self), fields(self.base_url, self.sender))]
    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<()> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: &self.sender,
            to: recipient,
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests;
