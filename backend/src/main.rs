mod service;
pub mod firefly {
    tonic::include_proto!("firefly");
}

use firefly::backend_server::BackendServer;
use firefly::service_client::ServiceClient;
use service::{BackendService, BackendState, Strip};
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tonic::transport::Server;
use tower_http::add_extension::AddExtensionLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let addr = "0.0.0.0:9000".parse()?;

    let mut strips: Vec<Strip> = Vec::new();

    get_strips(&mut strips).await;

    let backend_service = BackendService {};
    let state = BackendState { strips };

    Server::builder()
        .layer(AddExtensionLayer::new(Arc::new(Mutex::new(state))))
        .add_service(BackendServer::new(backend_service))
        .serve(addr)
        .await?;

    Ok(())
}

async fn get_strips(strips: &mut Vec<Strip>) {
    let mut has_more_strips = true;
    let mut strip_id = 1;

    while {
        let name = env::var(format!("LED_{strip_id}_NAME"));
        let endpoint = env::var(format!("LED_{strip_id}_ENDPOINT"));

        if name.is_err() || endpoint.is_err() {
            has_more_strips = false;
        }

        if let Ok(name) = name {
            if let Ok(endpoint) = endpoint {
                let client = ServiceClient::connect(format!("http://{endpoint}"))
                    .await
                    .unwrap_or_else(|_| panic!("Unable to connect to {}", endpoint));
                strips.push(Strip {
                    id: strip_id,
                    name,
                    client,
                });
            }
        }

        strip_id += 1;

        has_more_strips
    } {}
}
