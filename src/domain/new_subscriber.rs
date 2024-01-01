use anyhow::Result;

use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::routes::SubscribeFormData;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<SubscribeFormData> for NewSubscriber {
    type Error = anyhow::Error;

    fn try_from(value: SubscribeFormData) -> Result<Self> {
        let email = value.email.try_into()?;
        let name = value.name.try_into()?;

        Ok(Self { email, name })
    }
}
