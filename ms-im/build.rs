fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "proto/health.proto",
                "proto/im.proto",
                "../ms-identity/proto/identity.proto",
            ],
            &["proto", "../ms-identity/proto"],
        )?;
    Ok(())
}
