pub mod protobufs {
    include!("../../proto/generated/mod.rs");
    // // this was needed for grpc reflection api but not needed by default
    //    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    //        tonic::include_file_descriptor_set!("externalgrpc_descriptor");
}

use crate::libvirt::{
    create_instance, get_nodes_in_node_group, libvirt_delete_node, NODE_GROUP_REGEX,
};
use crate::node_template::generate_node_template;
use config::{Config, Value};
use core::time::Duration as sysDuration;
use once_cell::sync::OnceCell;
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
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::net::unix::SocketAddr;
use tokio::sync::Mutex;
use tokio::time::Duration as timeDuration;
use tonic::transport::Server;
use tonic::transport::{Identity, ServerTlsConfig};
use tonic::{Code, Request, Response, Status};

#[derive(Default)]
pub struct ImplementedNodeGroup {
    node_group: NodeGroup,
    options: NodeGroupAutoscalingOptions,
    node_template: Node,
}

#[derive(Default)]
pub struct ImplementedCloudProvider {
    config: RwLock<Config>,
    node_groups: Arc<Mutex<HashMap<String, ImplementedNodeGroup>>>,
    machine_count: Arc<Mutex<HashMap<String, i32>>>,
}
impl ImplementedCloudProvider {
    fn new(_config: Config) -> Self {
        let mut groupopts: HashMap<String, ImplementedNodeGroup> = HashMap::new();
        let nodegroups: Vec<Value> = _config.get_array("node_groups").unwrap();
        for ng in nodegroups {
            let table = match ng.into_table() {
                Ok(t) => t,
                Err(e) => {
                    panic!("Unable to parse node_groups into a table, can't proceed!");
                }
            };
            // create a NodeGroup object
            let id = table
                .get("name")
                .unwrap()
                .clone()
                .into_string()
                .expect("Couldn't read name of node group in node_groups");
            let min_size: i32 = table
                .get("min_nodes")
                .unwrap()
                .clone()
                .into_int()
                .unwrap_or(0) as i32;
            let max_size: i32 = table
                .get("max_nodes")
                .unwrap()
                .clone()
                .into_int()
                .unwrap_or(0) as i32;
            let node_group_object: NodeGroup = NodeGroup {
                id: id.clone(),
                min_size,
                max_size,
                debug: String::from(""),
            };
            // Create an AutoScalingOptions object
            let scale_down_utilization_threshold: f64 = table
                .get("scale_down_utilization_threshold")
                .unwrap()
                .clone()
                .into_float()
                .unwrap_or(0.0);
            let scale_down_gpu_utilization_threshold: f64 = table
                .get("scale_down_gpu_utilization_threshold")
                .unwrap()
                .clone()
                .into_float()
                .unwrap_or(0.0);
            let scale_down_unneeded_time_secs: i64 = table
                .get("scale_down_unneeded_after_secs")
                .unwrap()
                .clone()
                .into_int()
                .unwrap_or(300);
            let scale_down_unready_time_secs: i64 = table
                .get("scale_down_unready_after_secs")
                .unwrap()
                .clone()
                .into_int()
                .unwrap_or(300);
            let max_node_provision_time_secs: i64 = table
                .get("max_node_provisioning_time_secs")
                .unwrap()
                .clone()
                .into_int()
                .unwrap_or(300);
            let node_group_autoscaling_options: NodeGroupAutoscalingOptions =
                NodeGroupAutoscalingOptions {
                    scale_down_utilization_threshold,
                    scale_down_gpu_utilization_threshold,
                    scale_down_unneeded_time: Some(Duration {
                        duration: Some(
                            sysDuration::from_secs(scale_down_unneeded_time_secs as u64).as_nanos()
                                as i64,
                        ),
                    }),
                    scale_down_unready_time: Some(Duration {
                        duration: Some(
                            sysDuration::from_secs(scale_down_unready_time_secs as u64).as_nanos()
                                as i64,
                        ),
                    }),
                    max_node_provision_time: Some(Duration {
                        duration: Some(
                            sysDuration::from_secs(max_node_provision_time_secs as u64).as_nanos()
                                as i64,
                        ),
                    }),
                };

            let mut ing = ImplementedNodeGroup::default();
            ing.node_group = node_group_object;
            ing.options = node_group_autoscaling_options;
            ing.node_template = generate_node_template(table.clone());

            groupopts.insert(id, ing);
        }
        ImplementedCloudProvider {
            config: RwLock::new(_config),
            node_groups: Arc::new(Mutex::new(groupopts)),
            machine_count: Arc::new(Default::default()),
        }
    }
}

fn get_nodegroup_from_string(input: String) -> Option<String> {
    for cap in NODE_GROUP_REGEX.captures_iter(&input) {
        return Some(String::from(&cap[1]));
    }
    // this started out with an exclude list.  I'm wondering if I'll want to implement it again, so,
    // leaving this logic somewhat wonky rather than refactoring out of Option<String> for the meantime.
    Some(String::from(""))
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

        Ok(Response::new(resp))
    }

    async fn node_group_get_options(
        &self,
        _request: Request<NodeGroupAutoscalingOptionsRequest>,
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

pub async fn serve(_config: Config) -> Result<(), tonic::transport::Error> {
    let mut provider: ImplementedCloudProvider = ImplementedCloudProvider::new(_config);

    let settings = provider.config.read().unwrap().clone();

    let listen_addr = match settings.get_string("bind_addr") {
        Ok(s) => s,
        Err(e) => {
            panic!("bind_addr must be set either via config.toml or envvar.");
        }
    };
    let listen_port = match settings.get_int("bind_port") {
        Ok(s) => s,
        Err(e) => {
            panic!("bind_addr must be set either via config.toml or envvar.");
        }
    };
    let addr_string = format!("{}:{}", listen_addr, listen_port);
    let addr = match addr_string.parse() {
        Ok(a) => a,
        Err(e) => {
            panic!("Can't listen on {addr_string}: {e}")
        }
    };
    let tls = settings.get_bool("tls").unwrap_or(false);
    let mut cert: Option<String> = None;
    let mut key: Option<String> = None;

    let mut server = Server::builder();

    if tls {
        let cert_path = settings.get_string("cert_path").ok();
        let key_path = settings.get_string("key_path").ok();
        if cert_path.is_some() && key_path.is_some() {
            cert = std::fs::read_to_string(cert_path.unwrap()).ok();
            key = std::fs::read_to_string(key_path.unwrap()).ok();
            server = server
                .tls_config(
                    ServerTlsConfig::new()
                        .identity(Identity::from_pem(&cert.unwrap(), &key.unwrap())),
                )
                .unwrap();
        }
    }

    server
        .http2_keepalive_interval(Some(timeDuration::from_secs(1)))
        .add_service(CloudProviderServer::new(provider))
        .serve(addr)
        .await
}
