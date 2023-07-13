use crate::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::{
    NodeGroup, NodeGroupAutoscalingOptions,
};
use crate::cloud_provider_impl::protobufs::k8s::io::api::core::v1::Node;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const DEFAULT_NG_NAME: &str = "no-name-specified";
pub const DEFAULT_ARCH: &str = "amd64";
pub const DEFAULT_INSTANCE_TYPE: &str = "k3s";
pub const DEFAULT_OS: &str = "linux";
pub const DEFAULT_CPU: &str = "1";
pub const DEFAULT_MEM: &str = "512Mi";
pub const DEFAULT_PODS: &str = "110";

macro_rules! hashmap {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
}

#[derive(Default, Clone)]
pub struct ImplementedNodeGroup {
    node_group: NodeGroup,
    options: NodeGroupAutoscalingOptions,
    node_template: Node,
}

#[derive(Default)]
pub struct ImplementedCloudProvider {
    node_groups: Arc<Mutex<HashMap<String, ImplementedNodeGroup>>>,
    machine_count: Arc<Mutex<HashMap<String, i32>>>,
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
            model_node_cpu_count: Some(1),
            model_node_memory: Some("512Mi".to_string()),
            model_node_max_pods: Some(110),
            model_node_labels: hashmap! {
                "kubernetes.io/os" => DEFAULT_OS.to_string()
            },
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
