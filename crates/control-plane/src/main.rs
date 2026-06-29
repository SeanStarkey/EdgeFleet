use std::error::Error;

use tracing_subscriber::EnvFilter;

const BIND_ADDR_ENV: &str = "EDGEFLEET_BIND_ADDR";
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();

    let bind_addr = std::env::var(BIND_ADDR_ENV).unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!(bind_addr = %listener.local_addr()?, "control-plane listening");

    control_plane::serve(listener).await?;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("RUST_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
