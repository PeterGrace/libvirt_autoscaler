use crate::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::{
    NodeGroup, NodeGroupAutoscalingOptions,
};
use crate::cloud_provider_impl::protobufs::k8s::io::api::core::v1::Node;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const DEFAULT_NG_NAME: &str = "no-name-specified";
const DEFAULT_ARCH: &str = "amd64";
const DEFAULT_INSTANCE_TYPE: &str = "k3s";
const DEFAULT_OS: &str = "linux";
const DEFAULT_CPU: &str = "1";
const DEFAULT_MEM: &str = "512Mi";
const DEFAULT_PODS: &str = "110";

macro_rules! some_hashmap {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        Some(core::convert::From::from([$(($k, $v),)*]))
    }}
}

#[derive(Default, Clone)]
pub struct ImplementedNodeGroup {
    pub node_group: NodeGroup,
    pub options: NodeGroupAutoscalingOptions,
    pub node_template: Node,
}

#[derive(Default)]
pub struct ImplementedCloudProvider {
    pub node_groups: Arc<Mutex<HashMap<String, ImplementedNodeGroup>>>,
    pub machine_count: Arc<Mutex<HashMap<String, i32>>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeGroupOpts {
    name: String,
    model_node_cpu_count: Option<i32>,
    model_node_memory: Option<String>,
    model_node_max_pods: Option<i32>,
    model_node_labels: Option<HashMap<String, String>>,
    model_node_annotations: Option<HashMap<String, String>>,
    scale_down_utilization_threshold: Option<f64>,
    scale_down_gpu_utilization_threshold: Option<f64>,
    scale_down_unneeded_after_secs: Option<i64>,
    scale_down_unready_after_secs: Option<i64>,
    max_node_provisioning_time_secs: Option<i64>,
}

impl Default for NodeGroupOpts {
    fn default() -> Self {
        NodeGroupOpts {
            name: "".to_string(),
            model_node_cpu_count: Some(DEFAULT_CPU.parse().unwrap()),
            model_node_memory: Some(DEFAULT_MEM.to_string()),
            model_node_max_pods: Some(DEFAULT_PODS.parse().unwrap()),
            model_node_labels: Some(HashMap::from([(
                "kubernetes.io/os".to_string(),
                DEFAULT_OS.to_string(),
            )])),
            model_node_annotations: None,
            scale_down_utilization_threshold: None,
            scale_down_gpu_utilization_threshold: None,
            scale_down_unneeded_after_secs: None,
            scale_down_unready_after_secs: None,
            max_node_provisioning_time_secs: None,
        }
    }
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct AppConfig {}
