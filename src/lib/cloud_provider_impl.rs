use std::path::PathBuf;
use tonic::{Code, Request, Response, Status};
pub mod external_grpc {
    include!("../../proto/generated/mod.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

use external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_server::{
    CloudProvider, CloudProviderServer,
};
use tonic::transport::Server;
use tonic::transport::{Identity, ServerTlsConfig};

use crate::cloud_provider_impl::external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::Instance;
use external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::{
    CleanupRequest, CleanupResponse, GetAvailableGpuTypesRequest, GetAvailableGpuTypesResponse,
    GpuLabelRequest, GpuLabelResponse, NodeGroup, NodeGroupAutoscalingOptionsRequest,
    NodeGroupAutoscalingOptionsResponse, NodeGroupDecreaseTargetSizeRequest,
    NodeGroupDecreaseTargetSizeResponse, NodeGroupDeleteNodesRequest, NodeGroupDeleteNodesResponse,
    NodeGroupForNodeRequest, NodeGroupForNodeResponse, NodeGroupIncreaseSizeRequest,
    NodeGroupIncreaseSizeResponse, NodeGroupNodesRequest, NodeGroupNodesResponse,
    NodeGroupTargetSizeRequest, NodeGroupTargetSizeResponse, NodeGroupTemplateNodeInfoRequest,
    NodeGroupTemplateNodeInfoResponse, NodeGroupsRequest, NodeGroupsResponse,
    PricingNodePriceRequest, PricingNodePriceResponse, PricingPodPriceRequest,
    PricingPodPriceResponse, RefreshRequest, RefreshResponse,
};

use crate::libvirt::{get_node_groups, get_nodes_in_node_group, NODE_GROUP_REGEX};

#[derive(Default)]
pub struct ImplementedCloudProvider {}

fn get_nodegroup_from_string(input: String) -> Option<String> {
    for cap in NODE_GROUP_REGEX.captures_iter(&input) {
        return Some(String::from(&cap[1]));
    }
    None
}

#[tonic::async_trait]
impl CloudProvider for ImplementedCloudProvider {
    async fn node_groups(
        &self,
        _request: Request<NodeGroupsRequest>,
    ) -> std::result::Result<Response<NodeGroupsResponse>, Status> {
        let ng: Vec<NodeGroup> = match get_node_groups().await {
            Ok(n) => n,
            Err(_e) => {
                return Err(Status::new(Code::Unavailable, "error checking node groups"));
            }
        };

        let resp: NodeGroupsResponse = NodeGroupsResponse { node_groups: ng };
        Ok(Response::new(resp))
    }

    async fn node_group_for_node(
        &self,
        _request: Request<NodeGroupForNodeRequest>,
    ) -> std::result::Result<Response<NodeGroupForNodeResponse>, Status> {
        // this implementation may need additional logic as you can return null string as node group to indicate a node
        // should be ignored.
        let req = _request.into_inner();
        let mut response: NodeGroupForNodeResponse = NodeGroupForNodeResponse::default();
        let mut nodegroup: NodeGroup = NodeGroup::default();
        if let Some(node) = req.node {
            if let Some(node_group_name) = get_nodegroup_from_string(node.name) {
                nodegroup.id = node_group_name.clone();
                response.node_group = Some(nodegroup.clone());
                return Ok(Response::new(response));
            }
        }
        Err(tonic::Status::new(Code::NotFound, "node not found"))
    }

    async fn pricing_node_price(
        &self,
        _request: Request<PricingNodePriceRequest>,
    ) -> std::result::Result<Response<PricingNodePriceResponse>, Status> {
        Err(tonic::Status::new(
            Code::Unimplemented,
            "pricing_node_price not implemented",
        ))
    }

    async fn pricing_pod_price(
        &self,
        _request: Request<PricingPodPriceRequest>,
    ) -> std::result::Result<Response<PricingPodPriceResponse>, Status> {
        Err(tonic::Status::new(
            Code::Unimplemented,
            "pricing_pod_price not implemented",
        ))
    }

    async fn gpu_label(
        &self,
        _request: Request<GpuLabelRequest>,
    ) -> std::result::Result<Response<GpuLabelResponse>, Status> {
        let mut resp = GpuLabelResponse::default();
        resp.label = String::from("libvirt-autoscaler-gpu-type");
        Ok(Response::new(resp))
    }

    async fn get_available_gpu_types(
        &self,
        _request: Request<GetAvailableGpuTypesRequest>,
    ) -> std::result::Result<Response<GetAvailableGpuTypesResponse>, Status> {
        // return no gpus at the moment until/when I figure out how to do libvirt/kvm gpu passthrough
        let resp = GetAvailableGpuTypesResponse::default();
        Ok(Response::new(resp))
    }

    async fn cleanup(
        &self,
        _request: Request<CleanupRequest>,
    ) -> std::result::Result<Response<CleanupResponse>, Status> {
        Ok(Response::new(CleanupResponse::default()))
    }

    async fn refresh(
        &self,
        _request: Request<RefreshRequest>,
    ) -> std::result::Result<Response<RefreshResponse>, Status> {
        Ok(Response::new(RefreshResponse::default()))
    }

    async fn node_group_target_size(
        &self,
        _request: Request<NodeGroupTargetSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupTargetSizeResponse>, Status> {
        todo!()
    }

    async fn node_group_increase_size(
        &self,
        _request: Request<NodeGroupIncreaseSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupIncreaseSizeResponse>, Status> {
        todo!()
    }

    async fn node_group_delete_nodes(
        &self,
        _request: Request<NodeGroupDeleteNodesRequest>,
    ) -> std::result::Result<Response<NodeGroupDeleteNodesResponse>, Status> {
        todo!()
    }

    async fn node_group_decrease_target_size(
        &self,
        _request: Request<NodeGroupDecreaseTargetSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupDecreaseTargetSizeResponse>, Status> {
        todo!()
    }

    async fn node_group_nodes(
        &self,
        _request: Request<NodeGroupNodesRequest>,
    ) -> std::result::Result<Response<NodeGroupNodesResponse>, Status> {
        let req = _request.into_inner();
        let nodelist = match get_nodes_in_node_group(req.id).await {
            Ok(v) => v,
            Err(e) => {
                error!("Couldn't process node groups: {e}");
                return Err(tonic::Status::new(
                    Code::Unavailable,
                    "error in node_group_nodes",
                ));
            }
        };
        let instances: Vec<Instance> = nodelist
            .iter()
            .map(|nodename| Instance {
                id: nodename.to_owned(),
                status: None,
            })
            .collect();
        let mut response = NodeGroupNodesResponse::default();
        response.instances = instances;
        Ok(Response::new(response))
    }

    async fn node_group_template_node_info(
        &self,
        _request: Request<NodeGroupTemplateNodeInfoRequest>,
    ) -> std::result::Result<Response<NodeGroupTemplateNodeInfoResponse>, Status> {
        Err(tonic::Status::new(
            Code::Unimplemented,
            "node_group_template_node_info not implemented",
        ))
    }

    async fn node_group_get_options(
        &self,
        _request: Request<NodeGroupAutoscalingOptionsRequest>,
    ) -> std::result::Result<Response<NodeGroupAutoscalingOptionsResponse>, Status> {
        Err(tonic::Status::new(
            Code::Unimplemented,
            "node_group_get_options not implemented",
        ))
    }
}

pub async fn serve(
    _addr: String,
    _port: u16,
    cert_path: PathBuf,
    key_path: PathBuf,
) -> Result<(), tonic::transport::Error> {
    let addr = "[::]:50051".parse().unwrap();
    let foo: ImplementedCloudProvider = ImplementedCloudProvider::default();
    let cert = std::fs::read_to_string(cert_path).ok();
    let key = std::fs::read_to_string(key_path).ok();

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(external_grpc::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    debug!("pre-serve-builder.");
    let server = Server::builder()
        .tls_config(
            ServerTlsConfig::new().identity(Identity::from_pem(&cert.unwrap(), &key.unwrap())),
        )?
        .add_service(service)
        .add_service(CloudProviderServer::new(foo))
        .serve(addr)
        .await;
    debug!("post-serve-builder.");
    Ok(server.unwrap())
}
