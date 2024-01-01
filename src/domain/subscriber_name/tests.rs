use anyhow::Result;
use claims::assert_err;

use crate::domain::SubscriberName;

#[test]
fn a_256_grapheme_long_name_is_valid() -> Result<()> {
    let name = "ë".repeat(256);
    SubscriberName::try_from(name)?;

    Ok(())
}

#[test]
fn a_name_longer_than_256_graphemes_is_rejected() {
    let name = "ë".repeat(257);
    assert_err!(SubscriberName::try_from(name));
}

#[test]
fn whitespace_only_names_are_rejected() {
    let name = " ".to_string();
    assert_err!(SubscriberName::try_from(name));
}

#[test]
fn empty_string_is_rejected() {
    let name = "".to_string();
    assert_err!(SubscriberName::try_from(name));
}

#[test]
fn names_containing_an_invalid_character_are_rejected() {
    for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
        let name = name.to_string();
        assert_err!(SubscriberName::try_from(name));
    }
}

#[test]
fn a_valid_name_is_parsed_successfully() -> Result<()> {
    let name = "Ursula Le Guin".to_string();
    SubscriberName::try_from(name)?;

    Ok(())
}
