use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from("proto/generated");
    tonic_build::configure()
        .include_file("mod.rs")
        .out_dir(out_dir.clone())
        .file_descriptor_set_path(out_dir.join("externalgrpc_descriptor.bin"))
        .compile(
            &["proto/externalgrpc.proto"],
            &["proto/k8s.io", "proto/"]
        ).unwrap();

    let official_out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .include_file("mod.rs")
        .file_descriptor_set_path(official_out_dir.join("externalgrpc_descriptor.bin"))
        .compile(
            &["proto/externalgrpc.proto"],
            &["proto/k8s.io", "proto/"]
        ).unwrap();
}
