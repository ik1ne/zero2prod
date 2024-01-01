use anyhow::Result;

use crate::common::TestApp;

#[tokio::test]
async fn health_check_works() -> Result<()> {
    let test_app = TestApp::new().await?;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    Ok(())
}
