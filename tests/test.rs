#[cfg(test)]
#[macro_use]
extern crate tracing;
use std::path::PathBuf;
use tonic::{Request, Response, Status};
use libvirt_autoscaler::cloud_provider_impl::serve;
use crate::external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_client::CloudProviderClient;
use crate::external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::NodeGroupsRequest;

pub mod external_grpc {
    include!("../proto/generated/mod.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

const LISTEN_ADDR: &str="[::]";
const CONNECT_ADDR: &str="localhost";
const LISTEN_PORT: u16=50051;

#[ctor::ctor]
fn init() {
    let _ = dotenv::from_path("./.env");
    tracing_subscriber::fmt::init();
}


async fn setup()  {
    tokio::spawn(async move {
        let cert_path: PathBuf = PathBuf::from("tls/localhost.crt");
        let key_path: PathBuf = PathBuf::from("tls/localhost.key");
        serve(String::from(LISTEN_ADDR), LISTEN_PORT, cert_path, key_path).await;
    });
}

#[tokio::test]
async fn test_node_groups() {
    tokio::spawn(async move {
        let cert_path: PathBuf = PathBuf::from("tls/localhost.crt");
        let key_path: PathBuf = PathBuf::from("tls/localhost.key");
        serve(String::from(LISTEN_ADDR), LISTEN_PORT, cert_path, key_path).await;
    });
    let mut client = CloudProviderClient::connect(format!("https://{CONNECT_ADDR}:{LISTEN_PORT}")).await.unwrap();
    let request = tonic::Request::new(NodeGroupsRequest {});
    let rs = match client.node_groups(request).await {
        Ok(s) => s,
        Err(e) => {
            debug!("{:#?}", e);
            panic!("failed to get positive response");
        }
    };
    let node_groups = rs.get_ref();


    assert_eq!(node_groups.node_groups.len(), 0)
}
