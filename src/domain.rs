pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

pub struct SubscriberEmail(String);

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Self {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.len() > 256;

        const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|ch| FORBIDDEN_CHARACTERS.contains(&ch));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name", s);
        }

        Self(s)
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
