mod tests;

use std::str::FromStr;
use std::error::Error;

use bson::oid::ObjectId;

use crate::models::grpc::maestro_vault::{self, maestro_vault_service_server::MaestroVaultService};
use crate::stats;
use crate::filesystem;
use crate::models::users_disks::{ApproxUserDiskUpdate, ApproxUserDiskInfo, DiskAction};


#[derive(Debug, Default)]
pub struct MaestroVault {
    filesystem: Box<dyn filesystem::UserDiskFilesystem>,
}

impl MaestroVault {
  pub fn new() -> Result<MaestroVault, Box<dyn Error + Send>> {
    match filesystem::select_filesystem() {
      Ok(custom_fs) => {
        Ok(MaestroVault{filesystem: custom_fs})
      }
      Err(err) => {
        Err(err)
      }
    }
  }

  async fn update_logs(&self, file_id: &str, user_id: &str, disk_id: &str, action: DiskAction) {
      let db = stats::users_disks::MongoRepo::init().await;

      db.update_disk_logs(
        ApproxUserDiskUpdate{
          disk_id: ObjectId::from_str(disk_id).unwrap(),
          user_id: ObjectId::from_str(user_id).unwrap(),
          file_id: ObjectId::from_str(file_id).unwrap(),
          action: action
        },
        ApproxUserDiskInfo{
          disk_id: ObjectId::from_str(disk_id).unwrap(),
          user_id: ObjectId::from_str(user_id).unwrap()
        }
      ).await;
//
    // });
  }
}

/*
fn dir_exists(path: String) -> bool { // todo put that into filesystem
  // TODO create dir by maestro once,
  //  do not call this method everytime upload_file is called
  // todo make a card on improving the services coherency
  let resp = std::fs::create_dir(path);

  if resp.is_err() {
    return false;
  }

  return true;
} */

#[tonic::async_trait]
impl MaestroVaultService for MaestroVault {

  /* open and write */
  async fn upload_file(
      &self,
      request: tonic::Request<maestro_vault::UploadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFileStatus>, tonic::Status>
  // TODO mutualise code with function upload_files
  {
    // create directories for users with user_id
    // file is stored with the path : "user_id/file_id"
    // allowing easy browsing through users files (as they are all in the directory "users_id")
    let my_request: maestro_vault::UploadFileRequest = request.into_inner();
    // let my_path = my_request.user_id.as_str();

    // dir_exists(my_path.to_string()); // todo put that into filesystem
    // todo create a directory for all users directories, and subdirectories for organisations ?
    /*
     * TODO
     *  create a filepath builder, that is an interface,
     *  implement this interface for vault and safe (or name it vault-cache ?)
     *  with different filepath creation, depending on directories and symbolic lnks required
     *
     * choose interface at compile time or with env variable
     */
    let ret: Option<Box<dyn Error + Send>> = self.filesystem.create_file(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), my_request.content, my_request.store_type);
      // let ret = std::fs::write(my_path.to_string() + "/" + my_request.file_id.as_str(), my_request.content);

    match ret { // todo change return type or match branches
      Some(err) => {
        Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()))
      }
      None => {
        self.update_logs(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), DiskAction::CREATE).await;
        Ok(tonic::Response::new(maestro_vault::UploadFileStatus{}))
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
        let ret = self.filesystem.create_file(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), my_request.content, my_request.store_type);

    //   let ret = std::fs::write(String::from(my_request.user_id.as_str()) + "/" + my_request.file_id.as_str(), my_request.content);

        match ret {
            None => {
                self.update_logs(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), DiskAction::CREATE).await;
            }
            Some(err) => {
                eprintln!("{}", err); // todo does that work ?
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

    let ret = self.filesystem.remove_file(&my_request.file_id, &my_request.user_id, &my_request.disk_id);
    // let ret = std::fs::remove_file(String::from(my_request.user_id.as_str()) + "/" + my_request.file_id.as_str());

    match ret {
      None => {
        self.update_logs(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), DiskAction::DELETE).await;
      }
      Some(err) => {
        return Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()));
      }
    }
    return Ok(tonic::Response::new(status));
  }

  /* loop over unlink */
  async fn remove_files(
      &self,
      request: tonic::Request<maestro_vault::RemoveFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveFilesStatus>, tonic::Status>
  {
    let my_requests = request.into_inner();
    let mut status = maestro_vault::RemoveFilesStatus{file_id_failures: vec!()};

    for file_id in my_requests.file_id {
      let ret = self.filesystem.remove_file(&file_id, &my_requests.user_id, &my_requests.disk_id);

      // let ret = std::fs::remove_file(String::from(my_requests.user_id.as_str()) + "/" + file_id.as_str());

      match ret {
        None => {
          self.update_logs(file_id.as_str(), my_requests.user_id.as_str(), my_requests.disk_id.as_str(), DiskAction::DELETE).await;
        },
        Some(_) => {
          // todo print err
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

    let ret = self.filesystem.get_file_content(my_request.file_id.as_str());
    // let ret = std::fs::read(String::from(my_request.user_id.as_str()) + "/" + my_request.file_id.as_str());

    match ret {
      Ok(read_res) => {
        self.update_logs(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), DiskAction::READ).await;

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
      let ret = self.filesystem.get_file_content(file.file_id.as_str());
      // let ret = std::fs::read(String::from(file.user_id.as_str()) + "/" + file.file_id.as_str());

      match ret {
        Ok(read_res) => {
          self.update_logs(file.file_id.as_str(), file.user_id.as_str(), file.disk_id.as_str(), DiskAction::READ).await;

          status.files.push(maestro_vault::DownloadFilesElemStatus{file_id: file.file_id, content: read_res})
        },
        Err(_) => {
          // todo print err
          // todo add file_id to status
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }

  async fn get_files_meta_info(
        &self,
        request: tonic::Request<maestro_vault::GetFilesMetaInfoRequest>,
      ) -> Result<tonic::Response<maestro_vault::GetFilesMetaInfoStatus>, tonic::Status> {
        // todo after solving other probs
        let status = maestro_vault::GetFilesMetaInfoStatus{file: vec![]};

        return Ok(tonic::Response::new(status));
  }
}

