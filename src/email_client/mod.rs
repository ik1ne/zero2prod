use anyhow::Result;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

#[derive(Debug)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: String,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(url: String, sender: String, authorization_token: Secret<String>) -> Self {
        Self {
            http_client: Client::new(),
            base_url: url,
            sender,
            authorization_token,
        }
    }
    #[tracing::instrument(skip(self), fields(self.base_url, self.sender))]
    pub async fn send_email(
        &self,
        recipient: String,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<()> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.clone(),
            to: recipient.clone(),
            subject: subject.to_owned(),
            html_body: html_content.to_owned(),
            text_body: text_content.to_owned(),
        };

        self.http_client
            .post(url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests;
