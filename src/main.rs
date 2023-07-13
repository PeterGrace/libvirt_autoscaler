#[macro_use]
extern crate tracing;

use libvirt_autoscaler::cloud_provider_impl::serve;
use std::env;

#[tokio::main]
async fn main() {
    // setup logging
    let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
        Ok(s) => s,
        Err(_e) => "./config.toml".to_string(),
    };
    let settings = match config::Config::builder()
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
            panic!("{e}");
        }
    };
    match settings.get_string("log_level") {
        Ok(s) => env::set_var("RUST_LOG", s),
        Err(_) => env::set_var("RUST_LOG", "info"),
    }
    tracing_subscriber::fmt::init();

    info!(
        "libvirt_autoscaler {} {}",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    );
    let _ = serve().await;
}
