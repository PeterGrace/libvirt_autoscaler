use crate::cloud_provider_impl::protobufs::k8s::io::api::core::v1::{Node, NodeSpec, NodeStatus};
use crate::cloud_provider_impl::protobufs::k8s::io::apimachinery::pkg::api::resource::Quantity;
use crate::cloud_provider_impl::protobufs::k8s::io::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use config::{Config, Map, Value, ValueKind};
use std::sync::RwLock;

pub fn generate_node_template(ng: Map<String, Value>) -> Node {
    let mut node = Node::default();
    let mut node_spec = NodeSpec::default();
    let mut metadata = ObjectMeta::default();
    let fake_hostname = format!(
        "k8s-{}-new",
        ng.get("name")
            .unwrap()
            .clone()
            .into_string()
            .unwrap_or(DEFAULT_NG_NAME.to_string())
    );
    metadata.labels.insert(
        String::from("beta.kubernetes.io/arch"),
        ng.get("node-label-arch")
            .unwrap_or(&Value::new(
                None,
                ValueKind::String(DEFAULT_ARCH.parse().unwrap()),
            ))
            .clone()
            .into_string()
            .unwrap(),
    );
    metadata.labels.insert(
        String::from("beta.kubernetes.io/instance-type"),
        ng.get("node-label-instance-type")
            .unwrap_or(&Value::new(
                None,
                ValueKind::String(DEFAULT_INSTANCE_TYPE.parse().unwrap()),
            ))
            .clone()
            .into_string()
            .unwrap(),
    );
    metadata.labels.insert(
        String::from("beta.kubernetes.io/os"),
        ng.get("node-label-os")
            .unwrap_or(&Value::new(
                None,
                ValueKind::String(DEFAULT_OS.parse().unwrap()),
            ))
            .clone()
            .into_string()
            .unwrap(),
    );
    metadata.labels.insert(
        String::from("kubernetes.io/arch"),
        ng.get("node-label-arch")
            .unwrap_or(&Value::new(
                None,
                ValueKind::String(DEFAULT_ARCH.parse().unwrap()),
            ))
            .clone()
            .into_string()
            .unwrap(),
    );
    metadata.labels.insert(
        String::from("kubernetes.io/os"),
        ng.get("node-label-os")
            .unwrap_or(&Value::new(
                None,
                ValueKind::String(DEFAULT_OS.parse().unwrap()),
            ))
            .clone()
            .into_string()
            .unwrap(),
    );
    metadata.labels.insert(
        String::from("kubernetes.io/hostname"),
        fake_hostname.clone(),
    );
    metadata.labels.insert(
        String::from("node.kubernetes.io/instance-type"),
        ng.get("node-label-instance-type")
            .unwrap_or(&Value::new(
                None,
                ValueKind::String(DEFAULT_INSTANCE_TYPE.parse().unwrap()),
            ))
            .clone()
            .into_string()
            .unwrap(),
    );

    metadata.name = Some(fake_hostname.clone());
    node_spec.unschedulable = Some(false);
    let mut node_status = NodeStatus::default();
    node_status.allocatable.insert(
        String::from("cpu"),
        Quantity {
            string: Some(
                ng.get("node-cpu-count")
                    .unwrap_or(&Value::new(
                        None,
                        ValueKind::String(DEFAULT_CPU.parse().unwrap()),
                    ))
                    .clone()
                    .into_string()
                    .unwrap(),
            ),
        },
    );
    node_status.allocatable.insert(
        String::from("memory"),
        Quantity {
            string: Some(
                ng.get("node-memory")
                    .unwrap_or(&Value::new(
                        None,
                        ValueKind::String(DEFAULT_MEM.parse().unwrap()),
                    ))
                    .clone()
                    .into_string()
                    .unwrap(),
            ),
        },
    );
    node_status.allocatable.insert(
        String::from("pods"),
        Quantity {
            string: Some(
                ng.get("node-max-pods")
                    .unwrap_or(&Value::new(
                        None,
                        ValueKind::String(DEFAULT_PODS.parse().unwrap()),
                    ))
                    .clone()
                    .into_string()
                    .unwrap(),
            ),
        },
    );
    node_status.capacity.insert(
        String::from("cpu"),
        Quantity {
            string: Some(
                ng.get("node-cpu-count")
                    .unwrap_or(&Value::new(
                        None,
                        ValueKind::String(DEFAULT_CPU.parse().unwrap()),
                    ))
                    .clone()
                    .into_string()
                    .unwrap(),
            ),
        },
    );
    node_status.capacity.insert(
        String::from("memory"),
        Quantity {
            string: Some(
                ng.get("node-memory")
                    .unwrap_or(&Value::new(
                        None,
                        ValueKind::String(DEFAULT_MEM.parse().unwrap()),
                    ))
                    .clone()
                    .into_string()
                    .unwrap(),
            ),
        },
    );
    node_status.capacity.insert(
        String::from("pods"),
        Quantity {
            string: Some(
                ng.get("node-max-pods")
                    .unwrap_or(&Value::new(
                        None,
                        ValueKind::String(DEFAULT_PODS.parse().unwrap()),
                    ))
                    .clone()
                    .into_string()
                    .unwrap(),
            ),
        },
    );
    node.status = Some(node_status);
    node.spec = Some(node_spec);
    node.metadata = Some(metadata);
    return node;
}
