use std::path::PathBuf;
use tokio::time::sleep;
use tonic::{Code, Request, Response, Status};
pub mod protobufs {
    include!("../../proto/generated/mod.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_server::{
    CloudProvider, CloudProviderServer,
};
use tonic::transport::Server;
use tonic::transport::{Identity, ServerTlsConfig};
use tokio::time::Duration as timeDuration;
use protobufs::k8s::io::api::core::v1::{Node, NodeSpec, NodeStatus};
use protobufs::k8s::io::apimachinery::pkg::api::resource::Quantity;
use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::{Instance, NodeGroupAutoscalingOptions};
use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::{
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
use protobufs::k8s::io::apimachinery::pkg::apis::meta::v1::{Duration, ObjectMeta};
use crate::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::instance_status::InstanceState;
use crate::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::InstanceStatus;

use crate::libvirt::{
    create_instance, get_node_groups, get_nodes_in_node_group, libvirt_delete_node,
    NODE_GROUP_REGEX,
};

#[derive(Default)]
pub struct ImplementedCloudProvider {}

fn get_nodegroup_from_string(input: String) -> Option<String> {
    for cap in NODE_GROUP_REGEX.captures_iter(&input) {
        return Some(String::from(&cap[1]));
    }
    match input.as_str() {
        "k8snode05" => Some(String::from("static-asrock")),
        "k8snode06" => Some(String::from("static-asrock")),
        "k8snode07" => Some(String::from("static-asrock")),
        "tpi1n1" => Some(String::from("static-soquartz")),
        "tpi1n2" => Some(String::from("static-soquartz")),
        "tpi1n3" => Some(String::from("static-soquartz")),
        "tpi1n4" => Some(String::from("static-soquartz")),
        "tpi2n1" => Some(String::from("static-soquartz")),
        "tpi2n2" => Some(String::from("static-soquartz")),
        "tpi2n3" => Some(String::from("static-soquartz")),
        "tpi2n4" => Some(String::from("static-soquartz")),
        "nuc-k3smstr.g2.gfpd.us" => Some(String::from("static-nuc")),
        _ => None,
    }
}

#[tonic::async_trait]
impl CloudProvider for ImplementedCloudProvider {
    async fn node_groups(
        &self,
        _request: Request<NodeGroupsRequest>,
    ) -> std::result::Result<Response<NodeGroupsResponse>, Status> {
        // let ng: Vec<NodeGroup> = match get_node_groups().await {
        //     Ok(n) => n,
        //     Err(_e) => {
        //         return Err(Status::new(Code::Unavailable, "error checking node groups"));
        //     }
        // };
        let ng: Vec<NodeGroup> = vec![NodeGroup {
            id: "libvirt".to_string(),
            min_size: 0,
            max_size: 100,
            debug: "".to_string(),
        }];

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
            debug!("node_group_for_node: Looking up {}", node.name);
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
        request: Request<NodeGroupTargetSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupTargetSizeResponse>, Status> {
        let req: NodeGroupTargetSizeRequest = request.into_inner();
        let count = get_nodes_in_node_group(req.id.clone())
            .await
            .unwrap_or_else(|_| vec![])
            .len();
        let mut resp = NodeGroupTargetSizeResponse::default();
        resp.target_size = count as i32;
        debug!(
            "node group target size: Checking node group {}",
            req.id.clone()
        );
        if req.id.starts_with("static") {
            resp.target_size = 1;
        };
        Ok(Response::new(resp))
    }

    async fn node_group_increase_size(
        &self,
        request: Request<NodeGroupIncreaseSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupIncreaseSizeResponse>, Status> {
        let req = request.into_inner();
        let node_group = req.id;
        debug!(
            "Received request to scale node group {}, delta:{}",
            node_group.clone(),
            &req.delta
        );
        for n in 0..req.delta {
            match create_instance(node_group.clone()) {
                Ok(_) => {}
                Err(e) => {
                    error!("Couldn't create node in {node_group}: {e}");
                    return Err(Status::new(
                        Code::Aborted,
                        "Couldn't create instance in node_group {node_group",
                    ));
                }
            }
        }
        debug!("Should sleep now for 120 seconds to give nodes time to warm up");
        sleep(timeDuration::from_secs(12)).await;
        debug!("I should have waited for 120 seconds");
        let resp = NodeGroupIncreaseSizeResponse::default();
        Ok(Response::new(resp))
    }

    async fn node_group_delete_nodes(
        &self,
        request: Request<NodeGroupDeleteNodesRequest>,
    ) -> std::result::Result<Response<NodeGroupDeleteNodesResponse>, Status> {
        let req = request.into_inner();
        for node in req.nodes {
            match libvirt_delete_node(node.name.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Could not delete node {}: {e}", node.name.clone());
                    return Err(Status::new(
                        Code::Aborted,
                        format!("couldn't delete node {}: {e}", node.name),
                    ));
                }
            }
        }
        let resp = NodeGroupDeleteNodesResponse::default();
        Ok(Response::new(resp))
    }

    async fn node_group_decrease_target_size(
        &self,
        _request: Request<NodeGroupDecreaseTargetSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupDecreaseTargetSizeResponse>, Status> {
        Err(Status::new(
            Code::Unimplemented,
            "node_group_decrease_target_size",
        ))
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
                status: Some(InstanceStatus {
                    instance_state: i32::from(InstanceState::InstanceCreating),
                    error_info: None,
                }),
            })
            .collect();
        let mut response = NodeGroupNodesResponse::default();
        response.instances = instances;
        Ok(Response::new(response))
    }

    async fn node_group_template_node_info(
        &self,
        request: Request<NodeGroupTemplateNodeInfoRequest>,
    ) -> std::result::Result<Response<NodeGroupTemplateNodeInfoResponse>, Status> {
        let req = request.into_inner();
        debug!("{:#?}", req);
        let mut resp = NodeGroupTemplateNodeInfoResponse::default();
        let mut node = Node::default();
        let mut node_spec = NodeSpec::default();
        let mut metadata = ObjectMeta::default();
        let fake_hostname = format!("k8s-{}-new", req.id);
        metadata.labels.insert(
            String::from("beta.kubernetes.io/arch"),
            String::from("amd64"),
        );
        metadata.labels.insert(
            String::from("beta.kubernetes.io/instance-type"),
            String::from("k3s"),
        );
        metadata
            .labels
            .insert(String::from("beta.kubernetes.io/os"), String::from("linux"));
        metadata
            .labels
            .insert(String::from("kubernetes.io/arch"), String::from("amd64"));
        metadata
            .labels
            .insert(String::from("kubernetes.io/os"), String::from("linux"));
        metadata.labels.insert(
            String::from("kubernetes.io/hostname"),
            fake_hostname.clone(),
        );
        metadata.labels.insert(
            String::from("node.kubernetes.io/instance-type"),
            String::from("k3s"),
        );

        metadata.name = Some(fake_hostname.clone());
        node_spec.unschedulable = Some(false);
        node_spec.provider_id = Some(String::from("libvirt://{fake_hostname}"));
        let mut node_status = NodeStatus::default();
        node_status.allocatable.insert(
            String::from("cpu"),
            Quantity {
                string: Some(String::from("8")),
            },
        );
        node_status.allocatable.insert(
            String::from("memory"),
            Quantity {
                string: Some(String::from("8127028Ki")),
            },
        );
        node_status.allocatable.insert(
            String::from("pods"),
            Quantity {
                string: Some(String::from("110")),
            },
        );
        node_status.capacity.insert(
            String::from("cpu"),
            Quantity {
                string: Some(String::from("8")),
            },
        );
        node_status.capacity.insert(
            String::from("memory"),
            Quantity {
                string: Some(String::from("8127028Ki")),
            },
        );
        node_status.capacity.insert(
            String::from("pods"),
            Quantity {
                string: Some(String::from("110")),
            },
        );
        node.status = Some(node_status);
        node.spec = Some(node_spec);
        node.metadata = Some(metadata);
        //TODO: don't hardcode this
        match req.id.as_str() {
            "libvirt" => resp.node_info = Some(node),
            _ => {
                return Err(tonic::Status::new(
                    Code::NotFound,
                    format!("node info entry {} not found", req.id),
                ));
            }
        };
        Ok(Response::new(resp))
    }

    async fn node_group_get_options(
        &self,
        request: Request<NodeGroupAutoscalingOptionsRequest>,
    ) -> std::result::Result<Response<NodeGroupAutoscalingOptionsResponse>, Status> {
        let req = request.into_inner();
        let mut resp = NodeGroupAutoscalingOptionsResponse::default();
        let mut options = NodeGroupAutoscalingOptions::default();
        //TODO: make this configurable
        options.scale_down_gpu_utilization_threshold = 10 as f64;
        options.scale_down_utilization_threshold = 10 as f64;
        options.scale_down_unneeded_time = Some(Duration {
            duration: Some(300),
        });
        options.scale_down_unready_time = Some(Duration {
            duration: Some(600),
        });
        resp.node_group_autoscaling_options = Some(options);
        Ok(Response::new(resp))
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
        .register_encoded_file_descriptor_set(protobufs::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    debug!("pre-serve-builder.");
    let server = Server::builder()
        .tls_config(
            ServerTlsConfig::new().identity(Identity::from_pem(&cert.unwrap(), &key.unwrap())),
        )?
        .http2_keepalive_interval(Some(timeDuration::from_secs(1)))
        .add_service(service)
        .add_service(CloudProviderServer::new(foo))
        .serve(addr)
        .await;
    debug!("post-serve-builder.");
    Ok(server.unwrap())
}
