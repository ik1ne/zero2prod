use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<SubscribeFormData>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    let query_result = query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(connection.get_ref())
    .await;

    match query_result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
