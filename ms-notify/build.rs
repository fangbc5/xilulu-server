fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "../ms-identity/proto/device.proto",
                "../ms-im/proto/im.proto",
            ],
            &["../ms-identity/proto", "../ms-im/proto"],
        )?;
    Ok(())
}
