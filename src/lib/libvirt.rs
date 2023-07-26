use crate::cloud_provider_impl::protobufs::clusterautoscaler::cloudprovider::v1::externalgrpc::NodeGroup;
use crate::structs::ImplementedNodeGroup;
use crate::xml_consts::{VM_XML, VOL_XML};
use crate::SETTINGS;
use anyhow::{bail, Result};
use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;
use virt::connect::Connect;
use virt::domain::Domain;
use virt::storage_pool::StoragePool;
use virt::storage_vol::StorageVol;
use virt::sys;

lazy_static! {
    pub static ref NODE_GROUP_REGEX: Regex = Regex::new("k8s-(.+?)-.*").unwrap();
}

pub async fn get_node_groups() -> Result<Vec<NodeGroup>> {
    let conn = match connect_libvirt() {
        Some(conn) => conn,
        None => {
            bail!("Couldn't connect to libvirt!");
        }
    };

    match libvirt_node_groups(&conn) {
        Ok(v) => {
            disconnect(conn);
            info!("Found {} node groups.", v.len());
            Ok(v)
        }
        Err(e) => {
            disconnect(conn);
            bail!("failure in converting libvirt domain list to nodegroups: {e}");
        }
    }
}

pub async fn libvirt_delete_node(node_name: String) -> Result<()> {
    let conn = match connect_libvirt() {
        Some(conn) => conn,
        None => {
            bail!("Couldn't connect to libvirt!");
        }
    };
    let domain = match Domain::lookup_by_name(&conn, &node_name) {
        Ok(d) => d,
        Err(e) => {
            disconnect(conn);
            bail!("Couldn't find domain {}: {e}", node_name);
        }
    };
    if let Err(e) = domain.destroy() {
        bail!("Attempted to delete {} but received error: {e}", node_name);
    }
    if let Err(e) = domain.undefine() {
        info!(
            "{} wasn't able to be undefined, this is probably ok, however.  Error: {e}",
            node_name
        );
    }
    let image_pool = SETTINGS
        .read()
        .unwrap()
        .get_string("kvm_storage_pool")
        .unwrap();
    let default_pool = match StoragePool::lookup_by_name(&conn, &image_pool) {
        Ok(d) => d,
        Err(e) => {
            disconnect(conn);
            bail!("Can't find default storage pool: {e}");
        }
    };
    let store =
        match StorageVol::lookup_by_name(&default_pool, format!("{}.qcow2", node_name).as_str()) {
            Ok(s) => s,
            Err(e) => {
                disconnect(conn);
                bail!("Couldn't get storage volume for node {}: {e}", node_name);
            }
        };
    if let Err(e) = store.delete(0) {
        disconnect(conn);
        bail!(
            "We couldn't delete the storage volume for node {}: {e}",
            node_name
        );
    };
    Ok(())
}

pub async fn get_nodes_in_node_group(node_group: &ImplementedNodeGroup) -> Result<Vec<String>> {
    let conn = match connect_libvirt() {
        Some(conn) => conn,
        None => {
            bail!("Couldn't connect to libvirt!");
        }
    };
    let node_group_name = &node_group.node_group.id;
    match libvirt_get_nodes(&conn) {
        Ok(v) => {
            let mut node_list: Vec<String> = vec![];
            for node in v {
                for cap in NODE_GROUP_REGEX.captures_iter(&node) {
                    if node_group_name.eq_ignore_ascii_case(&cap[1]) {
                        node_list.push(node.clone());
                    }
                }
            }
            disconnect(conn);
            Ok(node_list)
        }
        Err(e) => {
            disconnect(conn);
            bail!("failure in retrieving node names: {e}");
        }
    }
}

fn connect_libvirt() -> Option<Connect> {
    let virt_uri = SETTINGS.read().unwrap().get_string("kvm_uri").unwrap();
    debug!("About to connect to {virt_uri}");
    let conn = match Connect::open(&virt_uri) {
        Ok(c) => c,
        Err(e) => {
            error!("Can't connect to libvirt: {e}");
            return None;
        }
    };
    Some(conn)
}

fn libvirt_get_nodes(conn: &Connect) -> Result<Vec<String>> {
    let flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE;
    let mut node_list: Vec<String> = vec![];
    if let Ok(doms) = conn.list_all_domains(flags) {
        for dom in doms {
            match dom.get_name() {
                Ok(s) => node_list.push(s),
                Err(_e) => {
                    warn!("The domain had no name: {}", dom.get_id().unwrap());
                }
            }
        }
        return Ok(node_list);
    } else {
        bail!("Unable to get node list.");
    };
}

fn libvirt_node_groups(conn: &Connect) -> Result<Vec<NodeGroup>> {
    let _flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE;
    let mut node_group_list: Vec<NodeGroup> = vec![];
    let node_list = match libvirt_get_nodes(&conn) {
        Ok(v) => v,
        Err(e) => {
            bail!("Unable to get node list: {e}");
        }
    };
    for node in node_list {
        for cap in NODE_GROUP_REGEX.captures_iter(&node) {
            node_group_list.push(NodeGroup {
                id: String::from(&cap[1]),
                ..Default::default()
            })
        }
    }
    Ok(node_group_list)
}

fn disconnect(mut conn: Connect) {
    if let Err(e) = conn.close() {
        error!("Failed to disconnect from libvirt: {e}");
    };
    debug!("Disconnected from libvirt");
}

pub async fn create_instance(node_group: &ImplementedNodeGroup) -> Result<()> {
    let conn = match connect_libvirt() {
        Some(conn) => conn,
        None => {
            bail!("Couldn't connect to libvirt!");
        }
    };
    debug!("Getting storage pool...");
    let image_pool = SETTINGS
        .read()
        .unwrap()
        .get_string("kvm_storage_pool")
        .unwrap();
    let default_pool = match StoragePool::lookup_by_name(&conn, &image_pool) {
        Ok(d) => d,
        Err(e) => {
            disconnect(conn);
            bail!("Can't find default storage pool: {e}");
        }
    };

    // decide on host details
    let uuid = Uuid::new_v4();
    let hostname = format!("k8s-{}-{uuid}", node_group.node_group.id);
    let networkname = String::from("bridged-vlan-3");

    // create disk
    debug!("Creating disk...");
    let volxml = String::from(VOL_XML).replace("HOSTNAME", &hostname);
    if let Err(e) = StorageVol::create_xml(&default_pool, &volxml, 0) {
        disconnect(conn);
        bail!("we attempted to create storage but failed: {e}");
    }

    // since we already have the node template built, we can use this rather than a lot of match calls to get the
    // values out of the config toml.
    let node_status = node_group.node_template.status.clone().unwrap();
    let node_cpu = node_status
        .allocatable
        .get("cpu")
        .unwrap()
        .string
        .clone()
        .unwrap();
    let node_mem = node_status
        .allocatable
        .get("memory")
        .unwrap()
        .string
        .clone()
        .unwrap();
    // create vm
    debug!("Creating vm...");
    let vmxml = String::from(VM_XML)
        .replace("HOSTNAME", &hostname)
        .replace("NUMCPUS", &node_cpu)
        .replace("MEMORYKB", &node_mem)
        .replace("NETWORK-NAME", &networkname);
    if let Err(e) = Domain::create_xml(&conn, &vmxml, 0) {
        disconnect(conn);
        bail!("Couldn't create VM: {e}");
    }
    disconnect(conn);
    info!("Successfully submitted libvirt domain {hostname} for startup");
    Ok(())
}
