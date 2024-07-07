use std::io::Read;

use proto::{
    vm_runtime_server::{VmRuntime, VmRuntimeServer},
    ExecuteRequest,
};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("vm_runtime");

    // for dev
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("vm_runtime_descriptor");
}

#[derive(Debug, Default)]
struct ZokratesService {}

#[tonic::async_trait]
impl VmRuntime for ZokratesService {
    async fn create(
        &self,
        request: tonic::Request<proto::CreateRequest>,
    ) -> Result<tonic::Response<proto::CreateResponse>, tonic::Status> {
        println!("received create");
        match convert_zlib_hex_to_string(&request.into_inner().content) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("{}", e),
        };

        let response = proto::CreateResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn execute_operator(
        &self,
        request: tonic::Request<ExecuteRequest>,
    ) -> Result<tonic::Response<proto::ExecuteResponse>, tonic::Status> {
        println!("received exe");
        let response = proto::ExecuteResponse {
            result: "asdf".into(),
        };
        Ok(tonic::Response::new(response))
    }
}

fn convert_zlib_hex_to_string(hex_str: &str) -> Result<String, String> {
    let bytes = match hex::decode(hex_str) {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("couldn't convert hex string to bytes {}", e)),
    };

    let mut d = flate2::read::ZlibDecoder::new(bytes.as_slice());

    let mut s = String::new();

    match d.read_to_string(&mut s) {
        Ok(_) => {}
        Err(e) => return Err(format!("couldn't decompress string {}", e)),
    };

    Ok(s)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:4004".parse()?;

    println!("starting grpc server on {}", addr);

    let zok = ZokratesService::default();

    // for dev
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(VmRuntimeServer::new(zok))
        .add_service(reflection) // for dev
        .serve(addr)
        .await?;

    Ok(())
}
