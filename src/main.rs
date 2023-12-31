use anyhow::Result;
use zero2prod::configuration::get_configuration;

use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = get_configuration()?;

    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = std::net::TcpListener::bind(address)?;

    run(listener)?.await?;

    Ok(())
}
