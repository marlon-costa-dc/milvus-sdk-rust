use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new("src/proto");

    tonic_prost_build::configure()
        .build_server(false)
        .out_dir(out_dir)
        .compile_protos(
            &[
                "milvus-proto/proto/common.proto",
                "milvus-proto/proto/milvus.proto",
                "milvus-proto/proto/schema.proto",
            ],
            &["milvus-proto/proto"],
        )?;

    // Post-process: rename RPC method `connect()` to `connect_rpc()` in the
    // MilvusService client to avoid E0592 conflict with tonic's transport
    // `connect<D>(dst)` method on the concrete Channel impl.
    let milvus_rs = out_dir.join("milvus.proto.milvus.rs");
    if milvus_rs.exists() {
        let content = fs::read_to_string(&milvus_rs)?;
        let fixed = content.replace(
            "pub async fn connect(\n            &mut self,\n            request: impl tonic::IntoRequest<super::ConnectRequest>,",
            "pub async fn connect_rpc(\n            &mut self,\n            request: impl tonic::IntoRequest<super::ConnectRequest>,",
        );
        if fixed != content {
            fs::write(&milvus_rs, fixed)?;
        }
    }

    Ok(())
}
