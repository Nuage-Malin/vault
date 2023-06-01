fn main() -> Result<(), Box<dyn std::error::Error>> {
	let protobufs_dir = "third_parties/protobuf-interfaces";

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .type_attribute(".", "#[derive(serde::Deserialize, serde::Serialize)]")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Duration", "::prost_wkt_types::Duration")
		// .out_dir(protobufs_dir.to_string() + "/generated")
        .compile(
            &[protobufs_dir.to_string() + "/src/Maestro_Vault/Maestro_Vault.proto"],
            &[protobufs_dir.to_string() + "/src"],

        )
        .unwrap_or_else(|e| panic!("compilation failed: {}", e));
    Ok(())
}
