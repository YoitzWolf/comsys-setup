fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(">> Build run");
    /*rust_grpc_web::configure()
        .compile(&["proto/auth.proto"], &["proto"])?;*/
    tonic_build::configure()
        .out_dir("./src/grpc/")
        .build_server(false)
        .compile(
            &["./proto/auth.proto"],
            &["../proto"],
        )
        .unwrap();
    Ok(())
}