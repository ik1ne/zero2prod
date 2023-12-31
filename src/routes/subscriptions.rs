use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{Error, PgPool};
use tracing::error;
use uuid::Uuid;

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
    match insert_subscriber(form, pg_pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(?e, "Failed to execute query");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(skip_all)]
async fn insert_subscriber(
    form: web::Form<SubscribeFormData>,
    pg_pool: web::Data<PgPool>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
