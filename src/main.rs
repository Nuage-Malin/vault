mod models;
mod maestro;
use crate::models::grpc::maestro_vault::{maestro_vault_service_server::MaestroVaultServiceServer};
use tonic::transport::Server;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let address = "[::1]:8080".parse().unwrap();
	let maestroService = maestro::MaestroVault::default();

	Server::builder().add_service(MaestroVaultServiceServer::new(maestroService))
	  .serve(address)
	  .await?;
	Ok(())
}
