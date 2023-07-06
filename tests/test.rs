#[cfg(test)]
#[macro_use]
extern crate tracing;

use tonic::Code;
use libvirt_autoscaler::cloud_provider_impl::{ImplementedCloudProvider};
use libvirt_autoscaler::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_server::CloudProvider;
use libvirt_autoscaler::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::{ExternalGrpcNode, NodeGroupForNodeRequest, NodeGroupNodesRequest, NodeGroupsRequest};

#[ctor::ctor]
async fn init() {
    let _ = dotenv::from_path("./.env");
    tracing_subscriber::fmt::init();
}

// #[test]
// fn test_create_vm() {
//     if let Err(e) = create_instance() {
//         panic!("{e}");
//     }
//     assert_eq!(true,true);
//
// }

#[tokio::test]
async fn test_node_groups() {
    let obj: NodeGroupsRequest = NodeGroupsRequest::default();
    let req = tonic::Request::new(obj);
    let _inst: ImplementedCloudProvider = ImplementedCloudProvider::default();

    match _inst.node_groups(req).await {
        Ok(s) => s,
        Err(e) => {
            debug!("{:#?}", e);
            panic!("failed to get positive response");
        }
    };
}
#[tokio::test]
async fn test_node_group_for_node_none() {
    let mut obj: NodeGroupForNodeRequest = NodeGroupForNodeRequest::default();
    obj.node = None;
    let req = tonic::Request::new(obj);
    let _inst: ImplementedCloudProvider = ImplementedCloudProvider::default();

    match _inst.node_group_for_node(req).await {
        Ok(_) => {
            panic!("Got results when should have received none.");
        }
        Err(e) => {
            assert_eq!(e.code(), Code::NotFound);
        }
    };
}
#[tokio::test]
async fn test_node_group_for_node_some() {
    let mut obj: NodeGroupForNodeRequest = NodeGroupForNodeRequest::default();
    let mut egn: ExternalGrpcNode = ExternalGrpcNode::default();
    egn.name = String::from("k8s-nodegroup-12345");
    obj.node = Some(egn);
    let req = tonic::Request::new(obj);
    let _inst: ImplementedCloudProvider = ImplementedCloudProvider::default();

    match _inst.node_group_for_node(req).await {
        Ok(s) => {
            let resp = s.into_inner();
            assert_eq!(resp.node_group.unwrap().id, "nodegroup");
        }
        Err(e) => {
            debug!("{:#?}", e);
            panic!("failed to get positive response");
        }
    };
}
#[tokio::test]
async fn test_nodes_by_node_group_some() {
    let mut obj = NodeGroupNodesRequest::default();
    obj.id = String::from("invalidnodegroupname");
    let req = tonic::Request::new(obj);
    let _inst: ImplementedCloudProvider = ImplementedCloudProvider::default();

    match _inst.node_group_nodes(req).await {
        Ok(s) => {
            let resp = s.into_inner();
            assert_eq!(resp.instances.len(), 0);
        }
        Err(e) => {
            panic!("Err: {e}");
        }
    };
}
