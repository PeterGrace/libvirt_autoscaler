use regex::Regex;
use anyhow::{Result, bail};
use virt::connect::Connect;

use virt::sys;
use lazy_static::lazy_static;
use crate::cloud_provider_impl::external_grpc::clusterautoscaler::cloudprovider::v1::externalgrpc::{NodeGroup};

const MIN_SIZE: i32 = 1;
const MAX_SIZE: i32 = 10;
const VIRT_URI: &str = "qemu+ssh://root@10.174.5.25/system";

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
            info!("Found {} node groups.",v.len());
            Ok(v)
        },
        Err(e) => {
            disconnect(conn);
            bail!("failure in converting libvirt domain list to nodegroups: {e}");
        }
    }


}
pub async fn get_nodes_in_node_group(node_group: String) -> Result<Vec<String>> {
    let conn = match connect_libvirt() {
        Some(conn) => conn,
        None => {
            bail!("Couldn't connect to libvirt!");
        }
    };
    match libvirt_get_nodes(&conn) {
        Ok(v) => {
            let mut node_list:Vec<String> = vec![];
            for node in v {
                for cap in NODE_GROUP_REGEX.captures_iter(&node) {
                    if node_group.eq_ignore_ascii_case(&cap[1]) {
                        node_list.push(node.clone());
                    }
                }
            };
            disconnect(conn);
            Ok(node_list)
        },
        Err(e) => {
            disconnect(conn);
            bail!("failure in retrieving node names: {e}");
        }
    }


}

fn connect_libvirt() -> Option<Connect> {
    debug!("About to connect to {VIRT_URI}");
    let conn = match Connect::open(VIRT_URI) {
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
                Err(e) => {
                    warn!("The domain had no name: {}",dom.get_id().unwrap());
                }
            }
        }
        return Ok(node_list);
    } else {
        bail!("Unable to get node list.");
    };
}

fn libvirt_node_groups(conn: &Connect) -> Result<Vec<NodeGroup>> {

    let flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE;
    let mut node_group_list: Vec<NodeGroup> = vec![];
    let node_list = match libvirt_get_nodes(&conn) {
        Ok(v) => v,
        Err(e) => {
            bail!("Unable to get node list: {e}");
        }
    };
    for node in node_list {
        for cap in NODE_GROUP_REGEX.captures_iter(&node) {
            node_group_list.push(NodeGroup{
                id:String::from(&cap[1]),
                min_size: MIN_SIZE,
                max_size: MAX_SIZE,
                debug: String::from("false")
            })
        }
    };
    Ok(node_group_list)
}

fn disconnect(mut conn: Connect) {
    if let Err(e) = conn.close() {
        error!("Failed to disconnect from libvirt: {e}");
    };
    debug!("Disconnected from libvirt");
}