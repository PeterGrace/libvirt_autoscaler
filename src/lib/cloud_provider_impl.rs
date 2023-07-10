use core::time::Duration as sysDuration;
use std::collections::HashMap;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status};
pub mod protobufs {
    include!("../../proto/generated/mod.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

use crate::libvirt::{
    create_instance, get_nodes_in_node_group, libvirt_delete_node, NODE_GROUP_REGEX,
};
use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::cloud_provider_server::{
    CloudProvider, CloudProviderServer,
};
use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::instance_status::InstanceState;
use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::InstanceStatus;
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
use protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::{
    Instance, NodeGroupAutoscalingOptions,
};
use protobufs::k8s::io::api::core::v1::{Node, NodeSpec, NodeStatus};
use protobufs::k8s::io::apimachinery::pkg::api::resource::Quantity;
use protobufs::k8s::io::apimachinery::pkg::apis::meta::v1::{Duration, ObjectMeta};
use tokio::time::Duration as timeDuration;
use tonic::transport::Server;
use tonic::transport::{Identity, ServerTlsConfig};

#[derive(Default)]
pub struct ImplementedCloudProvider {
    machine_count: Arc<Mutex<HashMap<String, i32>>>,
}

fn get_nodegroup_from_string(input: String) -> Option<String> {
    for cap in NODE_GROUP_REGEX.captures_iter(&input) {
        return Some(String::from(&cap[1]));
    }
    let ignore = Some(String::from(""));
    match input.as_str() {
        "k8snode05" => ignore.clone(),
        "k8snode06" => ignore.clone(),
        "k8snode07" => ignore.clone(),
        "tpi1n1" => ignore.clone(),
        "tpi1n2" => ignore.clone(),
        "tpi1n3" => ignore.clone(),
        "tpi1n4" => ignore.clone(),
        "tpi2n1" => ignore.clone(),
        "tpi2n2" => ignore.clone(),
        "tpi2n3" => ignore.clone(),
        "tpi2n4" => ignore.clone(),
        "nuc-k3smstr.g2.gfpd.us" => ignore.clone(),
        _ => None,
    }
}

#[tonic::async_trait]
impl CloudProvider for ImplementedCloudProvider {
    async fn node_groups(
        &self,
        _request: Request<NodeGroupsRequest>,
    ) -> std::result::Result<Response<NodeGroupsResponse>, Status> {
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
        //TODO: make this more dynamic, also in get_node_groups logic
        let node_groups = vec!["libvirt"];
        for ng in node_groups {
            let current_count = get_nodes_in_node_group(String::from(ng))
                .await
                .unwrap_or_else(|_| vec![])
                .len() as i32;
            let machinecount = self.machine_count.lock().await;
            let requested_count = machinecount.get(ng).unwrap_or(&0);
            if requested_count > &current_count {
                for _n in current_count..requested_count + 1 {
                    match create_instance(String::from(ng)).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Couldn't create node in {ng}: {e}");
                        }
                    }
                }
            }
        }
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
            "node group target size: node group {} has {} size.",
            req.id.clone(),
            count
        );
        Ok(Response::new(resp))
    }

    async fn node_group_increase_size(
        &self,
        request: Request<NodeGroupIncreaseSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupIncreaseSizeResponse>, Status> {
        let req = request.into_inner();
        let node_group = req.id;
        info!(
            "Received request to scale node group {}, delta:{}",
            node_group.clone(),
            &req.delta
        );

        let mut machinecount = self.machine_count.lock().await;
        machinecount.insert(node_group, req.delta);
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
                Ok(_) => {
                    info!("Deleted node {}", node.name.clone());
                }
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
        request: Request<NodeGroupDecreaseTargetSizeRequest>,
    ) -> std::result::Result<Response<NodeGroupDecreaseTargetSizeResponse>, Status> {
        let req = request.into_inner();
        let node_group = req.id;
        info!(
            "Received request to decrease size of node group {}, delta:{}",
            node_group.clone(),
            &req.delta
        );

        let mut machinecount = self.machine_count.lock().await;
        machinecount.insert(node_group, req.delta);
        let resp = NodeGroupDecreaseTargetSizeResponse::default();
        Ok(Response::new(resp))
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
                id: format!("k3s://{}", nodename.to_owned()),
                status: Some(InstanceStatus {
                    instance_state: i32::from(InstanceState::InstanceRunning),
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
        let mut resp = NodeGroupAutoscalingOptionsResponse::default();
        let mut options = NodeGroupAutoscalingOptions::default();
        //TODO: make this configurable
        options.scale_down_gpu_utilization_threshold = 10 as f64;
        options.scale_down_utilization_threshold = 10 as f64;
        options.scale_down_unneeded_time = Some(Duration {
            duration: Some(sysDuration::from_secs(300).as_nanos() as i64),
        });
        options.scale_down_unready_time = Some(Duration {
            duration: Some(sysDuration::from_secs(180).as_nanos() as i64),
        });
        options.max_node_provision_time = Some(Duration {
            duration: Some(sysDuration::from_secs(180).as_nanos() as i64),
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

    let server = Server::builder()
        .tls_config(
            ServerTlsConfig::new().identity(Identity::from_pem(&cert.unwrap(), &key.unwrap())),
        )?
        .http2_keepalive_interval(Some(timeDuration::from_secs(1)))
        .add_service(service)
        .add_service(CloudProviderServer::new(foo))
        .serve(addr)
        .await;
    Ok(server.unwrap())
}
