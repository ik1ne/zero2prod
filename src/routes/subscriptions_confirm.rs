use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(skip_all)]
pub async fn confirm(_parameters: web::Path<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
