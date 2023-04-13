pub mod firefly {
    tonic::include_proto!("firefly");
}
mod led;
mod service;
mod strip;
use firefly::service_server::ServiceServer;
use service::{FireFlyState, ServiceService};
use std::{env, sync::Arc};
use strip::Strip;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tower_http::add_extension::AddExtensionLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let device = env::var("SPI_DEVICE").expect("SPI_DEVICE env var missing.");
    let freq = env::var("SPI_FREQ")
        .expect("SPI_FREQ env var missing.")
        .parse()?;
    let amount_of_leds = env::var("AMOUNT_OF_LEDS")
        .expect("AMOUNT_OF_LEDS env var missing.")
        .parse()?;

    let port = env::var("PORT").expect("PORT env var missing.");

    let strip = Strip::new(&device, freq, amount_of_leds).expect("Failed creating strip");

    let addr = format!("0.0.0.0:{port}").parse()?;

    let service_service = ServiceService {};
    let state = FireFlyState { strip };

    Server::builder()
        .layer(AddExtensionLayer::new(Arc::new(Mutex::new(state))))
        .add_service(ServiceServer::new(service_service))
        .serve(addr)
        .await?;

    Ok(())
}
