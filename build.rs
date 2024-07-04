use std::error::Error;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // for dev
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("vm_runtime_descriptor.bin"))
        .compile(&["proto/vm_runtime.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/vm_runtime.proto")?;

    Ok(())
}
