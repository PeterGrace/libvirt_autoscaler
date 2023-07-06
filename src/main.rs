#[macro_use]
extern crate tracing;
use libvirt_autoscaler::cloud_provider_impl::serve;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // setup logging
    info!("Starting libvirt-autoscaler.");
    let _ = dotenv::from_path("./.env");
    tracing_subscriber::fmt::init();
    let cert_path: PathBuf = PathBuf::from("tls/pgdev.crt");
    let key_path: PathBuf = PathBuf::from("tls/pgdev.key");

    let _ = serve(String::from("[::]"), 50051, cert_path, key_path).await;
}
