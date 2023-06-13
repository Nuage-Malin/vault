use syscalls::{Sysno, syscall, Errno};
use crate::models::grpc::maestro_vault::{self, maestro_vault_service_server::MaestroVaultService, RemoveFilesStatus};

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
    let my_request = request.into_inner();

    // create directories for users with user_id
    // file is stored with the path : "user_id/file_id"
    // allowing easy browsing through users files (as they are all in the directory "users_id")

    // todo create a directory for all users directories, and subdirectories for organisations ?
    let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::open, (my_request.user_id + "/" + my_request.file_id.as_str()).as_ptr(), 1 /* O_WRONLY */) };

    match ret {
      Ok(fd) => {
        let content_len = my_request.content.len();
        let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::write, fd, my_request.content.as_ptr() as *const _ /* todo understand this (taken from https://github.com/jasonwhite/syscalls/blob/main/tests/test_syscall.rs) */, content_len) };
        match ret {
          Ok(wrote_count) => {
            if wrote_count == content_len {
              return Ok(tonic::Response::new(maestro_vault::UploadFileStatus{}));
            } else {
              return Err(tonic::Status::new(tonic::Code::DataLoss, format!("Could write only {} over {} bytes", wrote_count, content_len)));
            }
          },
          Err(errno) => {
            return Err(tonic::Status::new(tonic::Code::PermissionDenied, errno.to_string()));
          }
            }
      },
      Err(errno) => {
        return Err(tonic::Status::new(tonic::Code::PermissionDenied, errno.to_string()));
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
      let file_id = my_request.file_id.clone();
      let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::open, (my_request.user_id + "/" + file_id.as_str()).as_ptr(), 1 /* todo fix O_WRONLY , todo add O_CREAT*/) };

      match ret {
        Ok(fd) => {
          let content_len = my_request.content.len();
          let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::write, fd, my_request.content.as_ptr() as *const _ /* todo understand this (taken from https://github.com/jasonwhite/syscalls/blob/main/tests/test_syscall.rs) */, content_len) };
          match ret {
            Ok(wrote_count) => {
              if wrote_count != content_len {
                status.file_id_failures.push(file_id.clone());
              }
            },
            Err(_) => {
              status.file_id_failures.push(file_id.clone());
            }
              }
        },
        Err(_) => {
          status.file_id_failures.push(file_id.clone());
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
    let my_request = request.into_inner();
    let mut status = maestro_vault::RemoveFilesStatus{file_id_failures: vec!()};

    for my_file_id in my_request.file_id {
      let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::unlink, my_file_id.as_ptr()) };

      match ret {
        Ok(ret_val) => {
          print!("ret_val : {}", ret_val);
          if ret_val != 0 {
            status.file_id_failures.push(my_file_id);
          }
        },
        Err(errno) => {
          print!("errno : {}", errno);
          // status.file_id_failures.push(my_file_id);
          // Err(tonic::Status::new(tonic::Code::Ok, "remove_files"))
        },
      }
    }
    Ok(tonic::Response::new(status))
  }
  /// Download

    /* open, read, return content */
    async fn download_file(
      &self,
      request: tonic::Request<maestro_vault::DownloadFileRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFileStatus>, tonic::Status>
  {
    let my_request = request.into_inner();
    let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::open, (my_request.user_id + "/" + my_request.file_id.as_str()).as_ptr(), O /* O_RDONLY */) };

    match ret {
      Ok(ret_val) => {
        let ret:Result<usize, Errno> = unsafe { syscall!(Sysno::read, (my_request.user_id + "/" + my_request.file_id.as_str()).as_ptr(), O /* O_RDONLY */) };

      },
      Err(errno) => {

      }
    }

    return Err(tonic::Status::new(tonic::Code::Ok, "download_file"));
  }

    /* loop over open and read, return content */
    async fn download_files(
      &self,
      request: tonic::Request<maestro_vault::DownloadFilesRequest>,
  ) -> Result<tonic::Response<maestro_vault::DownloadFilesStatus>, tonic::Status>
  {

    return Err(tonic::Status::new(tonic::Code::Ok, "download_files"));
  }
}
