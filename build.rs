use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // libvirt dynamic library
    println!("cargo:rustc-link-lib=dylib=virt");

    // git hash
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // should we generate protos
    let run_build = env::var("GENERATE_PROTOS")
        .map(|v| v == "1")
        .unwrap_or(false);

    if run_build {
        let out_dir = PathBuf::from("proto/generated");
        tonic_build::configure()
            .include_file("mod.rs")
            .out_dir(out_dir.clone())
            .file_descriptor_set_path(out_dir.join("externalgrpc_descriptor.bin"))
            .compile(&["proto/externalgrpc.proto"], &["proto/k8s.io", "proto/"])
            .unwrap();

        let official_out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        tonic_build::configure()
            .include_file("mod.rs")
            .file_descriptor_set_path(official_out_dir.join("externalgrpc_descriptor.bin"))
            .compile(&["proto/externalgrpc.proto"], &["proto/k8s.io", "proto/"])
            .unwrap();
    }
}
