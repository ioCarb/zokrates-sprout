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
        todo!()
    }
    async fn execute_operator(
        &self,
        request: tonic::Request<ExecuteRequest>,
    ) -> Result<tonic::Response<proto::ExecuteResponse>, tonic::Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:5000".parse()?;

    println!("starting server on {}", addr);

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
