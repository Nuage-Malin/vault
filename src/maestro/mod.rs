mod tests;
use std::error::Error;


use crate::models::grpc::maestro_vault::{StorageType, DownloadFilesElemStatus};
use crate::models::grpc::maestro_vault::{self, maestro_vault_service_server::MaestroVaultService};
use crate::{stats, my_eprintln};
use crate::filesystem;
use crate::models::users_disks::DiskAction;

pub fn i32_to_storage_type(enum_num: Option<i32>) -> StorageType {
  match enum_num {
    Some(enum_num) => {
      match enum_num {
        0 => StorageType::None,
        1 => StorageType::UploadQueue,
        2 => StorageType::DownloadQueue,
        _ => StorageType::None
      }
    }
    None => {
      StorageType::None
    }
  }
}

pub fn storage_type_to_i32(enum_num: Option<StorageType>) -> i32 {
  match enum_num {
    Some(enum_num) => {
      match enum_num {
        StorageType::None => 0,
        StorageType::UploadQueue => 1,
        StorageType::DownloadQueue => 2,
      }
    }
    None => {
      0
    }
  }
}

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

  async fn update_logs(&self, file_id: &str, user_id: Option<String>, disk_id: Option<String>, action: DiskAction) {
      let mut my_disk_id: Option<String> = None;
      let mut my_user_id: Option<String> = None;

      if user_id.is_some() {
        my_user_id = user_id;
      } else {
        match self.filesystem.get_file_user(&file_id) {
          Ok(file_user) => {
            my_user_id = Some(file_user);
          }
          Err(_) => {}
        }
      }
      if disk_id.is_some() {
        my_disk_id = disk_id;
      } else {
        match self.filesystem.get_file_disk(file_id) {
          Ok(file_disk) => {
            my_disk_id = Some(file_disk);
          }
          Err(_) => {}
        }
      }
      let db = stats::users_disks::MongoRepo::init().await;

      db.update_disk_logs(my_disk_id, my_user_id, file_id, action).await;
  }
}

#[tonic::async_trait]
impl MaestroVaultService for MaestroVault {

  /// open and write
  async fn upload_file(
      &self,
      request: tonic::Request<maestro_vault::UploadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFileStatus>, tonic::Status>
  // TODO mutualise code with function upload_files
  {
    my_eprintln!("Request: upload_file"); /* todo create procedure logger module */

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

    match self.filesystem.create_file(my_request.file_id.as_str(),
                                      my_request.user_id.as_str(),
                                      my_request.disk_id.as_str(),
                                      my_request.content,
                                      Some(i32_to_storage_type(my_request.store_type))) { // todo change return type or match branches
      Some(err) => {
        Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()))
      }
      None => {
        self.update_logs(my_request.file_id.as_str(), Some(my_request.user_id), Some(my_request.disk_id), DiskAction::CREATE).await;
        Ok(tonic::Response::new(maestro_vault::UploadFileStatus{}))
      }
    }
  }

  /// loop over open and write
  async fn upload_files(
      &self,
      request: tonic::Request<maestro_vault::UploadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::UploadFilesStatus>, tonic::Status>
  // TODO mutualise code with function upload_file
  {
    my_eprintln!("Request: upload_files"); /* todo create procedure logger module */

    let my_requests = request.into_inner();
    let mut status = maestro_vault::UploadFilesStatus{file_id_failures: vec!()};

    for my_request in my_requests.files {
        match self.filesystem.create_file(my_request.file_id.as_str(), my_request.user_id.as_str(), my_request.disk_id.as_str(), my_request.content, Some(i32_to_storage_type(my_request.store_type))) {
            None => {
                self.update_logs(my_request.file_id.as_str(), Some(my_request.user_id), Some(my_request.disk_id), DiskAction::CREATE).await;
            }
            Some(err) => {
                eprintln!("Line {} in {} : {}", line!(), file!(), err); // todo does that work ?
                status.file_id_failures.push(my_request.file_id)
            }
        }
    }
    return Ok(tonic::Response::new(status));
  }

  async fn modify_file(
    &self,
    request: tonic::Request<maestro_vault::ModifyFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::ModifyFileStatus>, tonic::Status>
  {
    my_eprintln!("Request: modify_file"); /* todo create procedure logger module */
    let my_request: maestro_vault::ModifyFileRequest = request.into_inner();

    match self.filesystem.set_file_content(&my_request.file_id, my_request.content) {
      None => {
        self.update_logs(&my_request.file_id, None, None, DiskAction::WRITE).await;
      }
      Some(err) => {
        return Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()));
      }
    }
    return Ok(tonic::Response::new(maestro_vault::ModifyFileStatus{}));
  }

  /// unlink
  async fn remove_file(
      &self,
      request: tonic::Request<maestro_vault::RemoveFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveFileStatus>, tonic::Status>
  {
    my_eprintln!("Request: remove_file"); /* todo create procedure logger module */

    let my_request: maestro_vault::RemoveFileRequest = request.into_inner();

    match self.filesystem.remove_file(&my_request.file_id) {
      None => {
        self.update_logs(my_request.file_id.as_str(), None, None, DiskAction::DELETE).await;
      }
      Some(err) => {
        return Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()));
      }
    }
    return Ok(tonic::Response::new(maestro_vault::RemoveFileStatus{}));
  }

  /// loop over unlink
  async fn remove_files(
      &self,
      request: tonic::Request<maestro_vault::RemoveFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveFilesStatus>, tonic::Status>
  {
    my_eprintln!("Request: remoeve_files"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let mut status = maestro_vault::RemoveFilesStatus{file_id_failures: vec!()};

    for file_id in my_request.file_ids {
      match self.filesystem.remove_file(&file_id) {
        None => {
          self.update_logs(&file_id, None, None, DiskAction::DELETE).await;
        },
        Some(err) => {
          my_eprintln!("{}", err.to_string());
          status.file_id_failures.push(file_id)
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }

  async fn remove_user(&self, request: tonic::Request<maestro_vault::RemoveUserRequest>,
  ) -> Result<tonic::Response<maestro_vault::RemoveUserStatus>, tonic::Status>
  {
    my_eprintln!("Request: remove_user"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let status = maestro_vault::RemoveUserStatus{};

    match self.filesystem.remove_user(&my_request.user_id) {
      Some(err) => {
        return Err(tonic::Status::new(tonic::Code::Aborted, err.to_string()));
      }
      None => {
      }
    }
    return Ok(tonic::Response::new(status))
  }

  /// Download

    /// open, read, return content
    async fn download_file(
      &self,
      request: tonic::Request<maestro_vault::DownloadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFileStatus>, tonic::Status>
  {
    my_eprintln!("Request: download_file"); /* todo create procedure logger module */

    let my_request = request.into_inner();

    match self.filesystem.get_file_content(my_request.file_id.as_str()) {
      Ok(read_res) => {
        self.update_logs(my_request.file_id.as_str(), None, None, DiskAction::READ).await;

        Ok(tonic::Response::new(maestro_vault::DownloadFileStatus{content: read_res}))
      },
      Err(err) => {
        Err(tonic::Status::new(tonic::Code::PermissionDenied, err.to_string()))
      }
    }
  }

    /// loop over open and read, return content
    async fn download_files(
      &self,
      request: tonic::Request<maestro_vault::DownloadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFilesStatus>, tonic::Status>
  {
    my_eprintln!("Request: download_files"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let mut status = maestro_vault::DownloadFilesStatus{files: vec!()};

    for file in my_request.files {
      match self.filesystem.get_file_content(file.file_id.as_str()) {
        Ok(read_res) => {
          self.update_logs(file.file_id.as_str(), None, None, DiskAction::READ).await;

          status.files.push(maestro_vault::DownloadFilesElemStatus{file_id: file.file_id, content: read_res})
        },
        Err(err) => {
          eprintln!("Line {} in {} : {}", line!(), file!(), err.to_string())
          // todo add file_id to status
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }

    async fn download_storage_type_files(
      &self,
      request: tonic::Request<maestro_vault::DownloadStorageTypeFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFilesStatus>, tonic::Status>
  {
    my_eprintln!("Request: download_storage_type_files"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let mut status = maestro_vault::DownloadFilesStatus{files: vec!()};

    match self.filesystem.get_store_type_files(i32_to_storage_type(Some(my_request.store_type))) {
      Ok(store_type_file_ids) => {
        for file_id in store_type_file_ids {
          match self.filesystem.get_file_content(&file_id) {
            Ok(content) => {
              let elem = DownloadFilesElemStatus{file_id: file_id, content: content};

              status.files.push(elem);
            }
            Err(err) => {
              return Err(tonic::Status::new(tonic::Code::NotFound, err.to_string()));
            }
          }
        }
      }
      Err(err) => {
        return Err(tonic::Status::new(tonic::Code::Aborted, err.to_string()));
      }
    }
    return Ok(tonic::Response::new(status));
  }

  async fn get_file_meta_info(
    &self,
    request: tonic::Request<maestro_vault::GetFileMetaInfoRequest>,
  ) -> Result<tonic::Response<maestro_vault::GetFileMetaInfoStatus>, tonic::Status> {
    my_eprintln!("Request: get_file_meta_info"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let mut status = maestro_vault::GetFileMetaInfoStatus{file: None};
    let disk: String;
    let user: String;
    let mut store_types: Vec<i32> = vec![];

    /* todo */
    match self.filesystem.get_file_disk(&my_request.file_id) {
      Ok(file_disk) => {
        disk = file_disk;
      }
      Err(err) => {
        return Err(tonic::Status::new(tonic::Code::NotFound, err.to_string()));
      }
    }
    match self.filesystem.get_file_user(&my_request.file_id) {
      Ok(file_user) => {
        user = file_user;
      }
      Err(err) => {
        return Err(tonic::Status::new(tonic::Code::NotFound, err.to_string()));
      }
    }
    match self.filesystem.get_file_store_types(&my_request.file_id) {
      Ok(file_store_types) => {
        for file_store_type in file_store_types {
          let store_type_num = storage_type_to_i32(Some(file_store_type));

          store_types.push(store_type_num);
        }
      }
      Err(err) => {
        return Err(tonic::Status::new(tonic::Code::NotFound, err.to_string()));
      }
    }
    let file = maestro_vault::FileMetaInfo{file_id: my_request.file_id.to_string(), user_id: user.to_string(), disk_id: disk.to_string(), store_types: store_types.clone()};
    status.file = Some(file);
    return Ok(tonic::Response::new(status));
  }

  /**
   * if file_id exists in request, get_file
   * if disk id get_disk_files, get_files
   * if user id get_user_files, get_files
   * otherwise get_files_disks
   * then filter only what's common between all collected files
  */
  async fn get_files_meta_info(
    &self,
    request: tonic::Request<maestro_vault::GetFilesMetaInfoRequest>,
  ) -> Result<tonic::Response<maestro_vault::GetFilesMetaInfoStatus>, tonic::Status> {
    my_eprintln!("Request: get_Files_meta_info"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let mut status = maestro_vault::GetFilesMetaInfoStatus{files: vec![]};

    let mut files: Vec<Vec<String>> = vec![];

    fn filter_common_entries<V>(vec1: &Vec<V>, vec2: &Vec<V>) -> Vec<V>
    where
        V: Clone,
        V: Eq
    {
        let mut result = Vec::new();

        for val in vec1.iter() {
            if vec2.contains(val) {
              result.push(val.clone());
            }
        }
        result
    }
    if let Some(user_id) = my_request.user_id {
      match self.filesystem.get_user_files(&user_id) /* todo replace with only getting the file_id */ {
        Ok(cur_user_files) => {
          let mut user_files: Vec<String> = vec![];

          for (file_id, _content) in cur_user_files {
            user_files.push(file_id);
          }
          files.push(user_files);
        }
        Err(err) => {
          return Err(tonic::Status::new(tonic::Code::Aborted, err.to_string()));
        }
      }
    }
    if let Some(disk_id) = my_request.disk_id {
      match self.filesystem.get_disk_files(&disk_id) /* todo replace with only getting the file_id */ {
        Ok(cur_disk_files) => {
          let mut disk_files: Vec<String> = vec![];

          for (file_id, _content) in cur_disk_files {
            disk_files.push(file_id.clone());
          }
          files.push(disk_files);
        }
        Err(err) => {
          return Err(tonic::Status::new(tonic::Code::Aborted, err.to_string()));
        }
      }
    }
    if let Some(store_type) = my_request.store_type {
      match self.filesystem.get_store_type_files(i32_to_storage_type(Some(store_type))) {
        Ok(store_type_files) => {
          files.push(store_type_files);
        }
        Err(err) => {
          return Err(tonic::Status::new(tonic::Code::Aborted, err.to_string()));
        }
      }

    }
    let last_files_index = files.len() - 1;
    for index in 0..last_files_index {
      files[index + 1] = filter_common_entries(&files[index], &files[index + 1]);
    }
    // status.file = files[last_files_index].clone();
    for file_id in files[last_files_index].clone() {
      let user_id: String;
      let disk_id: String;
      let mut store_types: Vec<i32> = vec![];

      match self.filesystem.get_file_user(&file_id) {
        Ok(user) => {
          user_id = user;
        }
        Err(err) => {
          eprintln!("Line {} in {} : {}", line!(), file!(), err.to_string());
          continue;
        }
      }
      match self.filesystem.get_file_disk(&file_id) {
        Ok(disk) => {
          disk_id = disk;
        }
        Err(err) => {
          eprintln!("Line {} in {} : {}", line!(), file!(), err.to_string());
          continue;
        }
      }
      match self.filesystem.get_file_store_types(&file_id) {
        Ok(storage_types) => {
          for storage_type in storage_types {
            store_types.push(storage_type_to_i32(Some(storage_type)));
          }
        }
        Err(err) => {
          eprintln!("Line {} in {} : {}", line!(), file!(), err.to_string());
          continue;
        }
      }

      status.files.push(maestro_vault::FileMetaInfo{file_id: file_id,
        user_id: user_id,
        disk_id: disk_id,
        store_types: store_types,});
    }

    return Ok(tonic::Response::new(status));
  }

  async fn get_files_disks(
    &self,
    request: tonic::Request<maestro_vault::GetFilesDisksRequest>,
  ) -> Result<tonic::Response<maestro_vault::GetFilesDisksStatus>, tonic::Status> {
    my_eprintln!("Request: get_files_disks"); /* todo create procedure logger module */

    let my_request = request.into_inner();
    let mut status = maestro_vault::GetFilesDisksStatus{disk_ids: vec![]};


    for file_id in my_request.file_ids {
      match self.filesystem.get_file_disk(&file_id) {
        Ok(disk_id) => {
          status.disk_ids.push(disk_id);
        }
        Err(err) => {
          // my_eprintln!(); // todo
          my_eprintln!("{}", err.to_string());
          status.disk_ids.push("".to_string());
        }
      }
    }
    return Ok(tonic::Response::new(status));
  }

}

