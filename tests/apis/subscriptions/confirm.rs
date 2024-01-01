use crate::common::TestApp;

use anyhow::Result;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() -> Result<()> {
    let app = TestApp::new().await?;

    let response = reqwest::Client::new()
        .post(&format!("{}/subscriptions/confirm", app.address))
        .send()
        .await?;

    assert_eq!(response.status().as_u16(), 400);

    Ok(())
}
