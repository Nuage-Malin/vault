mod tests;
use std::str::FromStr;

use bson::oid::ObjectId;

use crate::models::grpc::maestro_vault::{self, maestro_vault_service_server::MaestroVaultService};
use crate::stats;
use crate::models::users_disks::{ApproxUserDiskUpdate, ApproxUserDiskInfo, DiskAction};


#[derive(Debug, Default)]
pub struct MaestroVault {
}

impl MaestroVault {
  pub fn new() -> MaestroVault {
    MaestroVault {}
  }
}

fn dir_exists(path: String) -> bool {
  // TODO create dir by maestro once,
  //  do not call this method everytime upload_file is called
  // todo make a card on improving the services coherency
  let resp = std::fs::create_dir(path);

  if resp.is_err() {
    return false;
  }

  return true;
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

    let my_path = my_request.user_id.as_str();
    dir_exists(my_path.to_string());
    // todo create a directory for all users directories, and subdirectories for organisations ?
    let ret = std::fs::write(my_path.to_string() + "/" + my_request.file_id.as_str(), my_request.content);

    match ret {
      Ok(_) => {

        let db = stats::users_disks::MongoRepo::init().await;

        db.update_disk_logs(
          ApproxUserDiskUpdate{
            disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap(),
            file_id: ObjectId::from_str(my_request.file_id.as_str()).unwrap(),
            action: DiskAction::CREATE
          },
          ApproxUserDiskInfo{
            disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap()
          }
        ).await;
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
      let ret = std::fs::write(String::from(my_request.user_id.as_str()) + "/" + my_request.file_id.as_str(), my_request.content);

      match ret {
        Ok(_) => {
          let db = stats::users_disks::MongoRepo::init().await;

          db.update_disk_logs(
            ApproxUserDiskUpdate{
              disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
              user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap(),
              file_id: ObjectId::from_str(my_request.file_id.as_str()).unwrap(),
              action: DiskAction::CREATE
            },
            ApproxUserDiskInfo{
              disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
              user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap()
            }
          ).await;
        },
        Err(_) => {
          status.file_id_failures.push(my_request.file_id)
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }

  /* unlink */
  async fn remove_file(
      &self,
      request: tonic::Request<maestro_vault::RemoveFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveFileStatus>, tonic::Status>
  {
    let my_request: maestro_vault::RemoveFileRequest = request.into_inner();
    let status = maestro_vault::RemoveFileStatus{};

    let ret = std::fs::remove_file(String::from(my_request.user_id.as_str()) + "/" + my_request.file_id.as_str());

    match ret {
      Ok(_) => {
        let db = stats::users_disks::MongoRepo::init().await;

        db.update_disk_logs(
          ApproxUserDiskUpdate{
            disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap(),
            file_id: ObjectId::from_str(my_request.file_id.as_str()).unwrap(),
            action: DiskAction::DELETE
          },
          ApproxUserDiskInfo{
            disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap()
          }
        ).await;
      },
      Err(err) => {
        return Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()));
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
      let ret = std::fs::remove_file(String::from(my_requests.user_id.as_str()) + "/" + file_id.as_str());

      match ret {
        Ok(_) => {
          let db = stats::users_disks::MongoRepo::init().await;

          db.update_disk_logs(
            ApproxUserDiskUpdate{
              disk_id: ObjectId::from_str(my_requests.disk_id.as_str()).unwrap(),
              user_id: ObjectId::from_str(my_requests.user_id.as_str()).unwrap(),
              file_id: ObjectId::from_str(file_id.as_str()).unwrap(),
              action: DiskAction::DELETE
            },
            ApproxUserDiskInfo{
              disk_id: ObjectId::from_str(my_requests.disk_id.as_str()).unwrap(),
              user_id: ObjectId::from_str(my_requests.user_id.as_str()).unwrap()
            }
          ).await;
        },
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
    let ret = std::fs::read(String::from(my_request.user_id.as_str()) + "/" + my_request.file_id.as_str());

    match ret {
      Ok(read_res) => {
        let db = stats::users_disks::MongoRepo::init().await;

        db.update_disk_logs(
          ApproxUserDiskUpdate{
            disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap(),
            file_id: ObjectId::from_str(my_request.file_id.as_str()).unwrap(),
            action: DiskAction::DELETE
          },
          ApproxUserDiskInfo{
            disk_id: ObjectId::from_str(my_request.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(my_request.user_id.as_str()).unwrap()
          }
        ).await;
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
      let ret = std::fs::read(String::from(file.user_id.as_str()) + "/" + file.file_id.as_str());

      match ret {
        Ok(read_res) => {
          let db = stats::users_disks::MongoRepo::init().await;
        db.update_disk_logs(
          ApproxUserDiskUpdate{
            disk_id: ObjectId::from_str(file.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(file.user_id.as_str()).unwrap(),
            file_id: ObjectId::from_str(file.file_id.as_str()).unwrap(),
            action: DiskAction::READ
          },
          ApproxUserDiskInfo{
            disk_id: ObjectId::from_str(file.disk_id.as_str()).unwrap(),
            user_id: ObjectId::from_str(file.user_id.as_str()).unwrap()
          }
        ).await;
          status.files.push(maestro_vault::DownloadFilesElemStatus{file_id: file.file_id, content: read_res})
        },
        Err(_) => {
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }
}

