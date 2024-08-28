use std::error::Error;
use std::{env, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let protoc_path = Command::new("which")
        .arg("protoc-c")
        .output()
        .expect("Failed to execute `which protoc` command");

    let protoc_path = std::str::from_utf8(&protoc_path.stdout)
        .expect("Failed to convert output to string")
        .trim();

    env::set_var("PROTOC", protoc_path);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("vm_runtime_descriptor.bin"))
        .compile(&["proto/vm_runtime.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/vm_runtime.proto")?;

    Ok(())
}
