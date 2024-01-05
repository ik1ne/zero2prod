use actix_web::{web, HttpResponse};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Error, PgPool};
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

#[tracing::instrument(skip(pg_pool, email_client))]
pub async fn subscribe(
    form: web::Form<SubscribeFormData>,
    pg_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let Ok(new_subscriber) = NewSubscriber::try_from(form.0) else {
        return HttpResponse::BadRequest().finish();
    };

    if let Err(e) = insert_subscriber(&new_subscriber, &pg_pool).await {
        error!(?e, "Failed to execute query");
        return HttpResponse::InternalServerError().finish();
    }

    if let Err(e) = send_confirmation_email(&email_client, new_subscriber, &base_url.0).await {
        error!(?e, "Failed to send a confirmation email");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(new_subscriber: &NewSubscriber, pg_pool: &PgPool) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pg_pool)
    .await
    .map_err(|e| {
        error!(?e, "Failed to execute query");
        e
    })?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
) -> Result<()> {
    let confirmation_link = format!("{}/subscriptions/confirm/my_token", base_url);
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
