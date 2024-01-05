use actix_web::{web, HttpResponse};
use anyhow::{Context, Result};
use chrono::Utc;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::domain::NewSubscriber;
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;

#[derive(serde::Deserialize, Debug)]
pub struct SubscribeFormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(skip(pg_pool, email_client, base_url))]
pub async fn subscribe(
    form: web::Form<SubscribeFormData>,
    pg_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let Ok(new_subscriber) = NewSubscriber::try_from(form.0) else {
        return HttpResponse::BadRequest().finish();
    };

    if let Err(e) = subscribe_internal(new_subscriber, &pg_pool, &email_client, &base_url.0).await {
        error!(?e, "Failed to store new subscriber");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

async fn subscribe_internal(
    new_subscriber: NewSubscriber,
    pg_pool: &PgPool,
    email_client: &EmailClient,
    base_url: &str,
) -> Result<()> {
    let subscriber_uuid = insert_subscriber(&new_subscriber, pg_pool).await?;

    let token = generate_subscription_token();

    store_token(pg_pool, subscriber_uuid, &token).await?;

    send_confirmation_email(email_client, new_subscriber, base_url, &token).await?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(30)
        .collect()
}

async fn store_token(pg_pool: &PgPool, subscriber_uuid: Uuid, token: &String) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        token,
        subscriber_uuid
    )
    .execute(pg_pool)
    .await
    .context("Failed to store subscription token")?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(new_subscriber: &NewSubscriber, pg_pool: &PgPool) -> Result<Uuid> {
    let subscriber_uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_uuid,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pg_pool)
    .await?;

    Ok(subscriber_uuid)
}

#[tracing::instrument(skip_all)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<()> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );

    email_client
        .send_email(
            new_subscriber.email.as_ref(),
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                confirmation_link
            ),
        )
        .await
}
