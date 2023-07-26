#[macro_use]
extern crate tracing;
pub mod cloud_provider_impl;
pub mod libvirt;
mod node_template;
pub mod structs;
mod xml_consts;

use config::Config;
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    pub static ref SETTINGS: RwLock<Config> = RwLock::new({
        let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
            Ok(s) => s,
            Err(_e) => "./config.toml".to_string(),
        };
        let settings = match Config::builder()
            .add_source(config::File::with_name(&cfg_file))
            .add_source(
                config::Environment::with_prefix("LIBVIRT_AUTOSCALER")
                    .try_parsing(true)
                    .list_separator(","),
            )
            .build()
        {
            Ok(s) => s,
            Err(e) => {
                panic!("{}", e);
            }
        };
        settings
    });
}
