
#[cfg(test)]
mod tests {
    // use crate::maestro::MaestroVaultService;
    use crate::maestro::maestro_vault;
    use crate::maestro::MaestroVault;
    use crate::models::grpc::maestro_vault::maestro_vault_service_server::MaestroVaultService;

    pub const FILE_IDS: [&str; 6] = ["655ceb05ee2884fd5e168721", "655ceb05ee2884fd5e168722", "655ceb05ee2884fd5e168723", "655ceb05ee2884fd5e168724", "655ceb05ee2884fd5e168725", "655ceb05ee2884fd5e168726"];
    pub const FILE_CONTENTS: [&str; 6] = ["content of file 0", "content of file 1", "content of file 2", "content of file 3", "content of file 4", "content of file 5"];
    pub const USER_ID: &str = "655ceb05ee2884fd5e16872a";
    pub const DISK_ID: &str = "655ceb05ee2884fd5e16872b";

    #[tokio::test()]
    async fn _01_upload_file_test() {

        // todo insert diskwakeup manually in script launch_unit_tests

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
    async fn _02_upload_files_test() {
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
    async fn _03_modify_file_test() {
        let request_content = maestro_vault::ModifyFileRequest{
            file_id: FILE_IDS[0].to_string(),
            content: FILE_CONTENTS[1].to_string().into_bytes()
        };
        match MaestroVault::new() {
            Ok(vault) => {
                let request = tonic::Request::new(request_content);
                let result = vault.modify_file(request).await;

                match result {
                    Ok(_response) => {
                    }
                    Err(error) => {
                        eprintln!("\nError: {}", error);
                        assert!(false)
                    }
                }
            }
            Err(err) => {
                eprintln!("\nError: {}", err);
                assert!(false)
            }
        }
    }

    #[tokio::test]
    async fn _04_download_file_test() {
        let request_content = maestro_vault::DownloadFileRequest{
            file_id: FILE_IDS[0].to_string(),
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
                                assert_eq!(content, FILE_CONTENTS[1]);
                                /* contents 1 because has been modified from 0 to 1 by modify_file test */

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
    async fn _05_download_files_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let request_content = maestro_vault::DownloadFilesRequest{
                    files: vec![
                        maestro_vault::DownloadFileRequest{file_id: FILE_IDS[1].to_string()},
                        maestro_vault::DownloadFileRequest{file_id: FILE_IDS[2].to_string()}
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
    async fn _06_remove_file_test() {

        match MaestroVault::new() {
            Ok(vault) => {
                // Create file
                let request_content = maestro_vault::UploadFileRequest{
                    file_id: FILE_IDS[3].to_string(),
                    user_id: USER_ID.to_string(),
                    disk_id: DISK_ID.to_string(),
                    content: FILE_CONTENTS[3].to_string().into_bytes(),
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

                // Remove file
                let request_content = maestro_vault::RemoveFileRequest{
                    file_id: FILE_IDS[3].to_string(),
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
    async fn _07_remove_files_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                // Create files
                let request_content = maestro_vault::UploadFilesRequest{
                    files: vec![
                        maestro_vault::UploadFileRequest{
                            file_id: FILE_IDS[4].to_string(),
                            user_id: USER_ID.to_string(),
                            disk_id: DISK_ID.to_string(),
                            content: FILE_CONTENTS[4].to_string().into_bytes(),
                            store_type: None,
                        },
                        maestro_vault::UploadFileRequest{
                            file_id: FILE_IDS[5].to_string(),
                            user_id: USER_ID.to_string(),
                            disk_id: DISK_ID.to_string(),
                            content: FILE_CONTENTS[5].to_string().into_bytes(),
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

                // Remove files
                let request_content = maestro_vault::RemoveFilesRequest{
                    file_ids: vec![
                            FILE_IDS[4].to_string(),
                            FILE_IDS[5].to_string(),
                        ]
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

    #[tokio::test]
    async fn _10_remove_non_existing_user_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let rm_user_request = maestro_vault::RemoveUserRequest{
                    user_id: "cafe99999999999999999999".to_string()
                };
                let request = tonic::Request::new(rm_user_request);

                match vault.remove_user(request).await {
                    Ok(_) => {
                        eprintln!("\nError, removed non existing user without returning an error");
                        assert!(false);
                    },
                    Err(_) => {
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
    async fn _11_remove_user_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let rm_user_request = maestro_vault::RemoveUserRequest{
                    user_id: USER_ID.to_string()
                };
                let request = tonic::Request::new(rm_user_request);

                match vault.remove_user(request).await {
                    Ok(_) => { /* response has no content so no need to check it */
                        // Check if user files and directories still exist
                        let rm_user_request = maestro_vault::GetFilesMetaInfoRequest{
                            user_id: Some(USER_ID.to_string()),
                            disk_id: None,
                            store_type: None
                        };
                        let request = tonic::Request::new(rm_user_request);

                        match vault.get_files_meta_info(request).await {
                            Ok(status) => {
                                let get_files_meta_info_status = status.into_inner();

                                if !get_files_meta_info_status.files.is_empty() {
                                    assert!(false);
                                }
                            },
                            Err(_) => {
                                /* Should return error as user doesn't exist anymore */
                            }
                        }
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

    #[tokio::test]
    async fn _12_remove_already_removed_user_test() {
        match MaestroVault::new() {
            Ok(vault) => {
                let rm_user_request = maestro_vault::RemoveUserRequest{
                    user_id: USER_ID.to_string()
                };
                let request = tonic::Request::new(rm_user_request);
                match vault.remove_user(request).await {
                    Ok(_) => {
                        eprintln!("\nError, removed already removed user without returning an error");
                        assert!(false);
                    },
                    Err(_) => {
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