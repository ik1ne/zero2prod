use claims::assert_err;
use fake::locales;
use fake::locales::Data;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

use super::*;

#[derive(Debug, Clone)]
struct ValidEmailFixture(pub String);

impl Arbitrary for ValidEmailFixture {
    fn arbitrary(g: &mut Gen) -> Self {
        let username = g
            .choose(locales::EN::NAME_FIRST_NAME)
            .unwrap()
            .to_lowercase();
        let domain = g.choose(&["com", "net", "org"]).unwrap();
        Self(format!("{}@example.{}", username, domain))
    }
}

#[test]
fn empty_string_is_rejected() {
    let email = "".to_string();
    assert_err!(SubscriberEmail::try_from(email));
}

#[test]
fn email_missing_at_symbol_is_rejected() {
    let email = "ursuladomain.com".to_string();
    assert_err!(SubscriberEmail::try_from(email));
}

#[test]
fn email_missing_subject_is_rejected() {
    let email = "@domain.com".to_string();
    assert_err!(SubscriberEmail::try_from(email));
}

#[quickcheck]
fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> Result<()> {
    SubscriberEmail::try_from(valid_email.0)?;

    Ok(())
}
