fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(">> Build run");
    /*rust_grpc_web::configure()
        .compile(&["proto/auth.proto"], &["proto"])?;*/
    tonic_build::configure()
        .out_dir("./src/grpc/")
        .extern_path(
            ".google.protobuf.Timestamp",
            "::prost_wkt_types::Timestamp"
        )
        .build_client(true)
        .build_server(false)
        .type_attribute(".", r"#[derive(serde::Deserialize, serde::Serialize)]")
        //.proto_path("./proto")
        .compile(
            &[
                "auth.proto",
                "generic.proto",
                "comp.proto",
                "comp_handler.proto"
            ], &["./proto"],
        )?;
    Ok(())
}