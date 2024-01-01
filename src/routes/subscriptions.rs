use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{Error, PgPool};
use tracing::error;
use uuid::Uuid;

use crate::domain::NewSubscriber;

#[derive(serde::Deserialize, Debug)]
pub struct SubscribeFormData {
    email: String,
    name: String,
}

#[tracing::instrument(skip(pg_pool))]
pub async fn subscribe(
    form: web::Form<SubscribeFormData>,
    pg_pool: web::Data<PgPool>,
) -> HttpResponse {
    let Ok(email) = form.0.email.try_into() else {
        return HttpResponse::BadRequest().finish();
    };

    let Ok(name) = form.0.name.try_into() else {
        return HttpResponse::BadRequest().finish();
    };

    let new_subscriber = NewSubscriber { email, name };

    match insert_subscriber(new_subscriber, pg_pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(?e, "Failed to execute query");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(
    new_subscriber: NewSubscriber,
    pg_pool: web::Data<PgPool>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pg_pool.as_ref())
    .await
    .map_err(|e| {
        error!(?e, "Failed to execute query");
        e
    })?;

    Ok(())
}
