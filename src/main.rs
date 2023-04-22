mod models;
mod maestro;
use std::net::{SocketAddr, Ipv6Addr, SocketAddrV6};

use crate::models::grpc::maestro_vault::{maestro_vault_service_server::MaestroVaultServiceServer};
use tonic::transport::Server;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let address_str = std::env::var("VAULT_ADDRESS").expect("VAULT_ADDRESS not set.");
	let port_str = std::env::var("VAULT_PORT").expect("VAULT_PORT not set.");
	let address: Ipv6Addr = address_str.parse().expect("VAULT_ADDRESS has wrong format");
	let port: u16 = port_str.parse().expect("VAULT_PORT has wrong format");
	let vault_address: SocketAddr = SocketAddr::V6(SocketAddrV6::new(address, port, 0, 0));
	let my_maestro_service = maestro::MaestroVault::default();

	Server::builder().add_service(MaestroVaultServiceServer::new(my_maestro_service))
	  .serve(vault_address)
	  .await?;
	Ok(())
}
