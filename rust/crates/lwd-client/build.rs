fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc);

    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "../../../proto/walletrpc/service.proto",
                "../../../proto/walletrpc/compact_formats.proto",
            ],
            &["../../../proto/walletrpc"],
        )?;

    println!("cargo:rerun-if-changed=../../../proto/walletrpc/service.proto");
    println!("cargo:rerun-if-changed=../../../proto/walletrpc/compact_formats.proto");

    Ok(())
}
