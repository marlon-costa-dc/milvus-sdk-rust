use std::fs;
use std::path::PathBuf;

const PROTO_DIR: &str = "milvus-proto/proto";
const PROTO_FILES: &[&str] = &[
    "common.proto",
    "feder.proto",
    "milvus.proto",
    "msg.proto",
    "rg.proto",
    "schema.proto",
];
const PROTO_ENTRY_FILES: &[&str] = &["common.proto", "milvus.proto", "schema.proto"];

fn proto_path(file: &str) -> PathBuf {
    PathBuf::from(PROTO_DIR).join(file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for proto_file in PROTO_FILES {
        println!(
            "cargo:rerun-if-changed={}",
            proto_path(proto_file).display()
        );
    }

    if std::env::var_os("PROTOC").is_none() {
        let protoc = protoc_bin_vendored::protoc_bin_path()?;
        std::env::set_var("PROTOC", protoc);
    }

    let proto_entry_files: Vec<_> = PROTO_ENTRY_FILES
        .iter()
        .map(|proto_file| proto_path(proto_file))
        .collect();

    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(&proto_entry_files, &[PathBuf::from(PROTO_DIR)])?;

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?);

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

    let schema_rs = out_dir.join("milvus.proto.schema.rs");
    if schema_rs.exists() {
        let content = fs::read_to_string(&schema_rs)?;
        let fixed = content
            .replace(
                "\n    #[deprecated]\n    #[prost(bool, tag = \"3\")]\n    pub auto_id: bool,",
                "\n    #[prost(bool, tag = \"3\")]\n    pub auto_id: bool,",
            )
            .replace(
                "/// deprecated later, keep compatible with c++ part now",
                "/// keep compatible with server; no replacement field in proto yet",
            );
        if fixed != content {
            fs::write(&schema_rs, fixed)?;
        }
    }

    Ok(())
}
