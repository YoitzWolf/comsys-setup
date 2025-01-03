

fn main() -> Result<(), Box<dyn std::error::Error>> {

    tonic_build::configure()
        .out_dir("src/gen/")
        .extern_path(
            ".google.protobuf.Timestamp",
            "::prost_wkt_types::Timestamp"
        )
        .compile_well_known_types(true)
        .build_client(false)
        .build_server(true)
        .type_attribute(".", r"#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(&[
            "./../comsys-wasm/proto/auth.proto",
            "./../comsys-wasm/proto/generic.proto",
            "./../comsys-wasm/proto/comp.proto",
            "./../comsys-wasm/proto/users.proto",
            "./../comsys-wasm/proto/comp_handler.proto"
        ], &["./../comsys-wasm/proto/"])?;
    Ok(())
}