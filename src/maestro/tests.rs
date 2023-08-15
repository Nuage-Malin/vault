
#[cfg(test)]
mod tests {
    // use crate::maestro::MaestroVaultService;
    use crate::maestro::maestro_vault;
    use crate::maestro::MaestroVault;
    use crate::models::grpc::maestro_vault::maestro_vault_service_server::MaestroVaultService;

    pub const FILE_IDS: [&str; 3] = ["64781cdf3236c1aef30e6188", "64787e97f0c3a964940559b0", "64788b1bf0c3a964940559b1"];
    pub const FILE_CONTENTS: [&str; 3] = ["upload_file_test", "upload_files_test", "second_string"];
    pub const USER_ID: &str = "64781e773236c1aef30e6189";
    pub const DISK_ID: &str = "64781e803236c1aef30e618a";

    #[tokio::test()]
    async fn _1_upload_file_test() {

        match MaestroVault::new() {
            Ok(vault) => {
                let request_content = maestro_vault::UploadFileRequest{
                    file_id: FILE_IDS[0].to_string(),
                    user_id: USER_ID.to_string(),
                    disk_id: DISK_ID.to_string(),
                    content: FILE_CONTENTS[0].to_string().into_bytes(),
                    store_type: None,
                };
                let request = tonic::Request::new(request_content);

                match vault.upload_file(request).await {
                    Ok(_) => {}
                    Err(error) => {
                        // Print the error message for the Err variant
                        eprintln!("\nError: {}", error);
                        assert!(false)
                    }
                }
            }
            Err(error) => {
                eprintln!("\nError: {}", error);
                assert!(false)
            }
        }

    }

    #[tokio::test]
    async fn _2_upload_files_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let request_content = maestro_vault::UploadFilesRequest{
                    files: vec![
                        maestro_vault::UploadFileRequest{
                            file_id: FILE_IDS[1].to_string(),
                            user_id: USER_ID.to_string(),
                            disk_id: DISK_ID.to_string(),
                            content: FILE_CONTENTS[1].to_string().into_bytes(),
                            store_type: None,
                        },
                        maestro_vault::UploadFileRequest{
                            file_id: FILE_IDS[2].to_string(),
                            user_id: USER_ID.to_string(),
                            disk_id: DISK_ID.to_string(),
                            content: FILE_CONTENTS[2].to_string().into_bytes(),
                            store_type: None,
                        },
                    ]
                };
                let request = tonic::Request::new(request_content);
                let result = vault.upload_files(request).await;

                match result {
                    Ok(response) => {
                        let status = response.into_inner();
                            assert_eq!(status.file_id_failures.len(), 0)
                    }
                    Err(error) => {
                        eprintln!("\nError: {}", error);
                        assert!(false)
                    }
                }
            }
            Err(error) => {
                eprintln!("\nError: {}", error);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn _3_download_file_test() {
        let request_content = maestro_vault::DownloadFileRequest{
            file_id: FILE_IDS[0].to_string(),
            user_id: USER_ID.to_string(),
            disk_id: DISK_ID.to_string(),
        };
        match MaestroVault::new() {
            Ok(vault) => {
                let request = tonic::Request::new(request_content);
                let result = vault.download_file(request).await;

                match result {
                    Ok(response) => {
                        let status = response.into_inner();
                        let file_content = String::from_utf8(status.content);

                        match file_content {
                            Ok(content) => {
                                assert_eq!(content, FILE_CONTENTS[0]);

                            },
                            Err(error) => {
                                eprintln!("\nError: {}", error);
                                assert!(false)
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("\nError: {}", error);
                        assert!(false)
                    }
                }
            }
            Err(error) => {
                eprintln!("\nError: {}", error);
                assert!(false)

            }
        }
    }

    #[tokio::test]
    async fn _4_download_files_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let request_content = maestro_vault::DownloadFilesRequest{
                    files: vec![
                        maestro_vault::DownloadFileRequest{
                            file_id: FILE_IDS[1].to_string(),
                            user_id: USER_ID.to_string(),
                            disk_id: DISK_ID.to_string()},
                        maestro_vault::DownloadFileRequest{
                            file_id: FILE_IDS[2].to_string(),
                            user_id: USER_ID.to_string(),
                            disk_id: DISK_ID.to_string()}
                    ]
                };
                let request = tonic::Request::new(request_content);
                let result = vault.download_files(request).await;
                let mut count = 1;

                match result {
                    Ok(response) => {
                        let status = response.into_inner();

                        for file in status.files {
                            assert_eq!(FILE_IDS[count], file.file_id);

                            match String::from_utf8(file.content) {
                                Ok(content) => {
                                    assert_eq!(content, FILE_CONTENTS[count]);
                                },
                                Err(error) => {
                                    eprintln!("\nError: {}", error);
                                    assert!(false)
                                }
                            }
                        count += 1;
                    }

                    }
                    Err(error) => {
                        // Print the error message for the Err variant
                        eprintln!("\nError: {}", error);
                        assert!(false);
                    }
                }
            }
            Err(error) => {
                eprintln!("\nError: {}", error);
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn _5_remove_file_test() {

        match MaestroVault::new() {
            Ok(vault) => {
                let request_content = maestro_vault::RemoveFileRequest{
                    file_id: FILE_IDS[0].to_string(),
                    user_id: USER_ID.to_string(),
                    disk_id: DISK_ID.to_string()
                };
                let request = tonic::Request::new(request_content);

                match vault.remove_file(request).await {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("\nError: {}", error);
                        assert!(false);
                    }
                }
            }
            Err(error) => {
                eprintln!("\nError: {}", error);
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn _6_remove_files_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let request_content = maestro_vault::RemoveFilesRequest{
                    file_id: vec![
                            FILE_IDS[1].to_string(),
                            FILE_IDS[2].to_string(),
                        ],
                    user_id: USER_ID.to_string(),
                    disk_id: DISK_ID.to_string()
                };
                let request = tonic::Request::new(request_content);

                match vault.remove_files(request).await {
                    Ok(response) => {
                        let status = response.into_inner();

                        assert_eq!(status.file_id_failures.len(), 0);
                    },
                    Err(error) => {
                        eprintln!("\nError: {}", error);
                        assert!(false);
                    }
                }
            }
            Err(error) => {
                eprintln!("\nError: {}", error);
                assert!(false);
            }
        }
    }
}