// use syscalls::{Sysno, syscall, Errno};
use crate::models::grpc::maestro_vault::{self, maestro_vault_service_server::MaestroVaultService};

#[derive(Debug, Default)]
pub struct MaestroVault {
  pub fake_item: String,
}

#[tonic::async_trait]
impl MaestroVaultService for MaestroVault {

  /* loop over open and write */
  async fn upload_file(
      &self,
      request: tonic::Request<maestro_vault::UploadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFileStatus>, tonic::Status>
  // TODO mutualise code with function upload_files
  {
    // create directories for users with user_id
    // file is stored with the path : "user_id/file_id"
    // allowing easy browsing through users files (as they are all in the directory "users_id")
    let my_request = request.into_inner();

    // todo create a directory for all users directories, and subdirectories for organisations ?
    let ret = std::fs::write(my_request.user_id + "/" + my_request.file_id.as_str(), my_request.content);

    match ret {
      Ok(_) => {
        Ok(tonic::Response::new(maestro_vault::UploadFileStatus{}))
      },
      Err(err) => {
        Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()))
      }
    }
  }

  /* loop over open and write */
  async fn upload_files(
      &self,
      request: tonic::Request<maestro_vault::UploadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFilesStatus>, tonic::Status>
  // TODO mutualise code with function upload_file
  {
    let my_requests = request.into_inner();
    let mut status = maestro_vault::UploadFilesStatus{file_id_failures: vec!()};

    for my_request in my_requests.files {
      let ret = std::fs::write(my_request.user_id + "/" + my_request.file_id.as_str(), my_request.content);

      match ret {
        Ok(_) => {},
        Err(_) => {
          status.file_id_failures.push(my_request.file_id)
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }

  /* unlink */
  async fn remove_files(
      &self,
      request: tonic::Request<maestro_vault::RemoveFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveFilesStatus>, tonic::Status>
  {
    let my_requests = request.into_inner();
    let mut status = maestro_vault::RemoveFilesStatus{file_id_failures: vec!()};

    for file_id in my_requests.file_id {
      let ret = std::fs::remove_file(my_requests.disk_id.clone() + "/" + file_id.as_str());

      match ret {
        Ok(_) => {},
        Err(_) => {
          status.file_id_failures.push(file_id)
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }
  /// Download

    /* open, read, return content */
    async fn download_file(
      &self,
      request: tonic::Request<maestro_vault::DownloadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFileStatus>, tonic::Status>
  {
    let my_request = request.into_inner();
    let ret = std::fs::read(my_request.user_id + "/" + my_request.file_id.as_str());

    match ret {
      Ok(read_res) => {
        Ok(tonic::Response::new(maestro_vault::DownloadFileStatus{content: read_res}))
      },
      Err(err) => {
        Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()))
      }
    }
  }

    /* loop over open and read, return content */
    async fn download_files(
      &self,
      request: tonic::Request<maestro_vault::DownloadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFilesStatus>, tonic::Status>
  {
    let my_request = request.into_inner();
    let mut status = maestro_vault::DownloadFilesStatus{files: vec!()};

    for file in my_request.files {
      let ret = std::fs::read(file.user_id + "/" + file.file_id.as_str());

      match ret {
        Ok(read_res) => {
          status.files.push(maestro_vault::DownloadFilesElemStatus{file_id: file.file_id, content: read_res})
        },
        Err(_) => {
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }
}
