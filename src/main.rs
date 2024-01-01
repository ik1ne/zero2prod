use anyhow::Result;

use zero2prod::configuration::Settings;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info", std::io::stdout);
    init_subscriber(subscriber)?;

    let configuration = Settings::get_configuration()?;

    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
