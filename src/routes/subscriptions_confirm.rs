use actix_web::{web, HttpResponse};
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(skip_all)]
pub async fn confirm(parameters: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    confirm_inner(&parameters.subscription_token, &pool)
        .await
        .unwrap_or_else(|e| {
            error!(?e);
            HttpResponse::InternalServerError().finish()
        })
}

async fn confirm_inner(subscription_token: &str, pool: &PgPool) -> Result<HttpResponse> {
    let id = get_subscriber_id_from_token(subscription_token, pool)
        .await
        .context("Failed to retrieve subscriber ID from the database")?;

    confirm_subscriber(id, pool)
        .await
        .context("Failed to confirm the subscriber in the database")?;

    Ok(HttpResponse::Ok().finish())
}

async fn get_subscriber_id_from_token(subscription_token: &str, pool: &PgPool) -> Result<Uuid> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_one(pool)
    .await
    .context("Failed to query the database")?;

    Ok(result.subscriber_id)
}

async fn confirm_subscriber(subscriber_id: Uuid, pool: &PgPool) -> Result<()> {
    let result = sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .context("Failed to update the subscriber status in the database")?;

    match result.rows_affected() {
        0 => Err(anyhow!("No subscriber found with that ID")),
        1 => Ok(()),
        n => bail!("Updated more than one row: {}", n),
    }
}
