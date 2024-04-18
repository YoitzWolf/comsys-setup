

fn main() -> Result<(), Box<dyn std::error::Error>> {

    tonic_build::configure()
        .out_dir("src/gen/")
        .compile(&[
            "./../comsys-wasm/proto/auth.proto"
        ], &["./../comsys-wasm/proto/"]).unwrap();

    Ok(())
}