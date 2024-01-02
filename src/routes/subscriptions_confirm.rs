use actix_web::{web, HttpResponse};
use serde::Deserialize;

// TODO decide get or post
#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(skip_all)]
pub async fn confirm(_parameters: web::Json<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
