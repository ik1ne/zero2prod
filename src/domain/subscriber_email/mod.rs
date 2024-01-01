use anyhow::{ensure, Result};
use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl TryFrom<String> for SubscriberEmail {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        ensure!(
            validate_email(&value),
            "{} is not a valid email address",
            value
        );

        Ok(Self(value))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
