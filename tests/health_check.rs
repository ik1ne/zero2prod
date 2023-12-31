use anyhow::Result;

use common::{spawn_app, TestApp};

mod common;

#[tokio::test]
async fn health_check_works() -> Result<()> {
    let TestApp {
        address,
        db_pool: _,
    } = spawn_app().await?;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    Ok(())
}
