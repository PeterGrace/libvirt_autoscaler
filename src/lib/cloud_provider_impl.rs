use std::path::PathBuf;
use tonic::{Code, Request, Response, Status};
pub mod external_grpc {
    include!("../../proto/generated/mod.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

use external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_server::{
    CloudProvider,
    CloudProviderServer
};
use tonic::{
    transport::Server
};
use tonic::transport::{Identity, ServerTlsConfig};


use external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::{
    CleanupRequest,
    CleanupResponse,
    GetAvailableGpuTypesRequest,
    GetAvailableGpuTypesResponse,
    GpuLabelRequest,
    GpuLabelResponse,
    NodeGroupAutoscalingOptionsRequest,
    NodeGroupAutoscalingOptionsResponse,
    NodeGroupDecreaseTargetSizeRequest,
    NodeGroupDecreaseTargetSizeResponse,
    NodeGroupDeleteNodesRequest,
    NodeGroupDeleteNodesResponse,
    NodeGroupForNodeRequest,
    NodeGroupForNodeResponse,
    NodeGroupIncreaseSizeRequest,
    NodeGroupIncreaseSizeResponse,
    NodeGroupNodesRequest,
    NodeGroupNodesResponse,
    NodeGroupTargetSizeRequest,
    NodeGroupTargetSizeResponse,
    NodeGroupTemplateNodeInfoRequest,
    NodeGroupTemplateNodeInfoResponse,
    NodeGroupsRequest,
    NodeGroupsResponse,
    NodeGroup,
    PricingNodePriceRequest,
    PricingNodePriceResponse,
    PricingPodPriceRequest,
    PricingPodPriceResponse,
    RefreshRequest,
    RefreshResponse};

use crate::libvirt::{get_node_groups};

#[derive(Default)]
pub struct PGCloudProvider {}

#[tonic::async_trait]
impl CloudProvider for PGCloudProvider {
    async fn node_groups(&self, request: Request<NodeGroupsRequest>) -> std::result::Result<Response<NodeGroupsResponse>, Status> {

        let ng:Vec<NodeGroup> = match get_node_groups().await {
            Ok(n) => n,
            Err(e) => {
                return Err(Status::new(Code::Unavailable, "error checking node groups"));
            }
        };

        let resp :NodeGroupsResponse = NodeGroupsResponse { node_groups: ng};
        Ok(Response::new(resp))
    }

    async fn node_group_for_node(&self, request: Request<NodeGroupForNodeRequest>) -> std::result::Result<Response<NodeGroupForNodeResponse>, Status> {
        todo!()
    }

    async fn pricing_node_price(&self, request: Request<PricingNodePriceRequest>) -> std::result::Result<Response<PricingNodePriceResponse>, Status> {
        todo!()
    }

    async fn pricing_pod_price(&self, request: Request<PricingPodPriceRequest>) -> std::result::Result<Response<PricingPodPriceResponse>, Status> {
        todo!()
    }

    async fn gpu_label(&self, request: Request<GpuLabelRequest>) -> std::result::Result<Response<GpuLabelResponse>, Status> {
        todo!()
    }

    async fn get_available_gpu_types(&self, request: Request<GetAvailableGpuTypesRequest>) -> std::result::Result<Response<GetAvailableGpuTypesResponse>, Status> {
        todo!()
    }

    async fn cleanup(&self, request: Request<CleanupRequest>) -> std::result::Result<Response<CleanupResponse>, Status> {
        todo!()
    }

    async fn refresh(&self, request: Request<RefreshRequest>) -> std::result::Result<Response<RefreshResponse>, Status> {
        todo!()
    }

    async fn node_group_target_size(&self, request: Request<NodeGroupTargetSizeRequest>) -> std::result::Result<Response<NodeGroupTargetSizeResponse>, Status> {
        todo!()
    }

    async fn node_group_increase_size(&self, request: Request<NodeGroupIncreaseSizeRequest>) -> std::result::Result<Response<NodeGroupIncreaseSizeResponse>, Status> {
        todo!()
    }

    async fn node_group_delete_nodes(&self, request: Request<NodeGroupDeleteNodesRequest>) -> std::result::Result<Response<NodeGroupDeleteNodesResponse>, Status> {
        todo!()
    }

    async fn node_group_decrease_target_size(&self, request: Request<NodeGroupDecreaseTargetSizeRequest>) -> std::result::Result<Response<NodeGroupDecreaseTargetSizeResponse>, Status> {
        todo!()
    }

    async fn node_group_nodes(&self, request: Request<NodeGroupNodesRequest>) -> std::result::Result<Response<NodeGroupNodesResponse>, Status> {
        todo!()
    }

    async fn node_group_template_node_info(&self, request: Request<NodeGroupTemplateNodeInfoRequest>) -> std::result::Result<Response<NodeGroupTemplateNodeInfoResponse>, Status> {
        todo!()
    }

    async fn node_group_get_options(&self, request: Request<NodeGroupAutoscalingOptionsRequest>) -> std::result::Result<Response<NodeGroupAutoscalingOptionsResponse>, Status> {
        todo!()
    }
}

pub async fn serve(addr: String, port: u16, cert_path: PathBuf, key_path: PathBuf) -> Result<(),tonic::transport::Error> {
    let addr = "[::]:50051".parse().unwrap();
    let foo :PGCloudProvider = PGCloudProvider::default();
    let cert = std::fs::read_to_string(cert_path).ok();
    let key = std::fs::read_to_string(key_path).ok();

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(external_grpc::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .tls_config(ServerTlsConfig::new()
            .identity(Identity::from_pem(&cert.unwrap(), &key.unwrap()))
        )?
        .add_service(service)
        .add_service(CloudProviderServer::new(foo))
        .serve(addr)
        .await

}