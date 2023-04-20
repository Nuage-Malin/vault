
use crate::models::grpc::maestro_vault::{self, maestro_vault_service_server::MaestroVaultService};

#[derive(Debug, Default)]
pub struct MaestroVault {
  pub fake_item: String,
}

#[tonic::async_trait]
impl MaestroVaultService for MaestroVault {

  async fn upload_file(
      &self,
      request: tonic::Request<maestro_vault::UploadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFileStatus>, tonic::Status>
  {
    return Err(tonic::Status::new(tonic::Code::Ok, "upload_file"));
  }

  async fn upload_files(
      &self,
      request: tonic::Request<maestro_vault::UploadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFilesStatus>, tonic::Status>
  {
    return Err(tonic::Status::new(tonic::Code::Ok, "upload_files"));
  }

  async fn remove_files(
      &self,
      request: tonic::Request<maestro_vault::RemoveFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveFilesStatus>, tonic::Status>
  {
    return Err(tonic::Status::new(tonic::Code::Ok, "remove_files"));
  }
  /// Download

  async fn download_file(
      &self,
      request: tonic::Request<maestro_vault::DownloadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFileStatus>, tonic::Status>
  {
    return Err(tonic::Status::new(tonic::Code::Ok, "download_file"));
  }

  async fn download_files(
      &self,
      request: tonic::Request<maestro_vault::DownloadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFilesStatus>, tonic::Status>
  {
    return Err(tonic::Status::new(tonic::Code::Ok, "download_files"));
  }
}
