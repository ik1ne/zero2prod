use anyhow::{bail, Result};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        let is_empty_or_whitespace = value.trim().is_empty();

        let is_too_long = value.graphemes(true).count() > 256;

        const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            value.chars().any(|ch| FORBIDDEN_CHARACTERS.contains(&ch));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            bail!("{} is not a valid subscriber name", value);
        }

        Ok(Self(value))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
