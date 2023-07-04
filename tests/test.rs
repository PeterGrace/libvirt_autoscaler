#[cfg(test)]
#[macro_use]
extern crate tracing;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tokio::runtime::Builder;
use tokio::task;
use tokio_test;
use tonic::{Request, Response, Status};

use libvirt_autoscaler::cloud_provider_impl::serve;

use crate::external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_client::CloudProviderClient;

use crate::external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::{ExternalGrpcNode, NodeGroupForNodeRequest, NodeGroupsRequest};

pub mod external_grpc {
    include!("../proto/generated/mod.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

const LISTEN_ADDR: &str="[::]";
const CONNECT_ADDR: &str="localhost";
const LISTEN_PORT: u16=50051;


#[ctor::ctor]
async fn init() {
    let _ = dotenv::from_path("./.env");
    tracing_subscriber::fmt::init();
    // Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap()
    //     .spawn (async {
    //         let cert_path: PathBuf = PathBuf::from("tls/localhost.crt");
    //         let key_path: PathBuf = PathBuf::from("tls/localhost.key");
    //         debug!("Starting server!");
    //         serve(String::from(LISTEN_ADDR), LISTEN_PORT, cert_path, key_path).await;
    //     });
    tokio_test::task::spawn (async {
            let cert_path: PathBuf = PathBuf::from("tls/localhost.crt");
            let key_path: PathBuf = PathBuf::from("tls/localhost.key");
            debug!("Starting server!");
            serve(String::from(LISTEN_ADDR), LISTEN_PORT, cert_path, key_path).await;
        });
    sleep(Duration::from_millis(3000)).unwrap();
    debug!("Hopefully server is ready after 3 secs!");

}

#[tokio::test]
async fn test_node_groups() {
    let mut client = CloudProviderClient::connect(format!("https://{CONNECT_ADDR}:{LISTEN_PORT}")).await.unwrap();
    let request = tonic::Request::new(NodeGroupsRequest {});
    let rs = match client.node_groups(request).await {
        Ok(s) => s,
        Err(e) => {
            debug!("{:#?}", e);
            panic!("failed to get positive response");
        }
    };
}

#[tokio::test]
async fn test_node_group_for_node_none() {
    let mut client = CloudProviderClient::connect(format!("https://{CONNECT_ADDR}:{LISTEN_PORT}")).await.unwrap();
    let request = tonic::Request::new(NodeGroupForNodeRequest { node: None });
    let rs = match client.node_group_for_node(request).await {
        Ok(s) => s,
        Err(e) => {
            debug!("{:#?}", e);
            panic!("failed to get positive response");
        }
    };
}
#[tokio::test]
async fn test_node_group_for_node_some() {
    let mut client = CloudProviderClient::connect(format!("https://{CONNECT_ADDR}:{LISTEN_PORT}")).await.unwrap();
    let request = tonic::Request::new(NodeGroupForNodeRequest {
        node: Some(ExternalGrpcNode {
            provider_id: "".to_string(),
            name: String::from("foobar"),
            labels: Default::default(),
            annotations: Default::default()
        })
    });
    let rs = match client.node_group_for_node(request).await {
        Ok(s) => s,
        Err(e) => {
            debug!("{:#?}", e);
            panic!("failed to get positive response");
        }
    };
}
