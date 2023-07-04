#[macro_use]
extern crate tracing;
use libvirt_autoscaler::cloud_provider_impl::serve;
use std::path::PathBuf;


#[tokio::main]
async fn main() {
    // setup logging
    let _ = dotenv::from_path("./.env");
    tracing_subscriber::fmt::init();
    let cert_path: PathBuf = PathBuf::from("tls/localhost.crt");
    let key_path: PathBuf = PathBuf::from("tls/localhost.key");

    serve(String::from("[::]"),
          50051,
          cert_path,
          key_path).await;
}
