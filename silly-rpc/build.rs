fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("src/proto/btc_constants.proto")?;
    Ok(())
}

/*
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["src/proto/btc_constants.proto"], &["src/proto"])?;
    Ok(())
}
*/
