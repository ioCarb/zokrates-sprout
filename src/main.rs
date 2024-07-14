mod compute_witness;
mod generate_proof;
mod utils;

use compute_witness::compute_witness_wrapper;
use generate_proof::compute_proof_wrapper;
use lazy_static::lazy_static;
use log::{info, trace};
use proto::{
    vm_runtime_server::{VmRuntime, VmRuntimeServer},
    ExecuteRequest,
};

use std::{collections::HashMap, io::Cursor};
use tokio::sync::Mutex;
use tonic::transport::Server;
use utils::{convert, group_by};

mod proto {
    tonic::include_proto!("vm_runtime");

    // for dev
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("vm_runtime_descriptor");
}

#[derive(Debug, Default)]
struct ZokratesService {}

// TODO: think about storing tasks decompressed and to a database
// lazy_static! {
//     static ref CONTENT_MAP: Mutex<HashMap<i32, Vec<u8>>> = Mutex::new(HashMap::new());
// }

lazy_static! {
    static ref PROJECT_MAP: Mutex<HashMap<u64, proto::CreateRequest>> = Mutex::new(HashMap::new());
}

#[tonic::async_trait]
impl VmRuntime for ZokratesService {
    async fn create(
        &self,
        request: tonic::Request<proto::CreateRequest>,
    ) -> Result<tonic::Response<proto::CreateResponse>, tonic::Status> {
        trace!("received CreateRequest");
        let request = request.into_inner();

        {
            let mut map = PROJECT_MAP.lock().await;
            map.insert(request.project_id, request);
        }

        let response = proto::CreateResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn execute_operator(
        &self,
        request: tonic::Request<ExecuteRequest>,
    ) -> Result<tonic::Response<proto::ExecuteResponse>, tonic::Status> {
        trace!("received ExecuteRequest");
        let request = request.into_inner();

        info!("datas: {:?}", request.datas);

        let content;
        let method;
        let proving_key;
        // let verify_key;

        {
            let create_request; // move out of lock scope???
            let map = PROJECT_MAP.lock().await;
            create_request = match map.get(&request.project_id) {
                Some(d) => d,
                None => {
                    return Err(tonic::Status::not_found(format!(
                        "couldn't find project_id: {}",
                        request.project_id
                    )))
                }
            };

            content = convert(&create_request.content, "content").await?;
            method = create_request.exp_params[0].clone();
            proving_key = convert(&create_request.exp_params[1], "proving_key").await?;
            // verify_key = convert(&create_request.exp_params[2], "verify_key").await?;
        }

        info!("received datas: {:?}", &request.datas);

        let content_cursor = Cursor::new(&content);

        // append message wise data1 data2 data3
        // let datas = request.datas.iter().flat_map(|s| s.split(' '));

        let datas = group_by(request.datas, ',').map_err(|e| {
            tonic::Status::invalid_argument(format!("some issue with the datas format {}", e))
        })?;

        info!("formatted datas: {}", &datas);

        let datas = datas.split(' ');

        let witness_reader = match compute_witness_wrapper(content_cursor, datas) {
            Ok(w) => w,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "couldn't deserialize program content {}",
                    e
                )))
            }
        };

        let content_cursor = Cursor::new(&content);

        let witness = Cursor::new(witness_reader);
        let proving_key = Cursor::new(proving_key);

        let proof = match compute_proof_wrapper(content_cursor, proving_key, witness, &method) {
            Ok(p) => p,
            Err(e) => {
                return Err(tonic::Status::internal(format!(
                    "couldnt compute proof {}",
                    e
                )))
            }
        };

        info!("proof: {}", proof.as_str());

        let response = proto::ExecuteResponse {
            result: proof.into(),
        };

        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = "0.0.0.0:4001".parse()?; // TODO: read port from env

    info!("starting grpc server on {}", addr);

    let zok = ZokratesService::default();

    // for dev
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    let max_message_size = 15 * 1042 * 1042 * 1024; // 15GB

    let vm_service = VmRuntimeServer::new(zok)
        .max_encoding_message_size(max_message_size)
        .max_decoding_message_size(max_message_size);

    Server::builder()
        .add_service(vm_service)
        .add_service(reflection)
        .serve(addr)
        .await?;

    Ok(())
}
