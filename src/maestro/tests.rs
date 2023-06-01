
#[cfg(test)]
mod tests {
    // use crate::maestro::MaestroVaultService;
    use crate::maestro::maestro_vault;
    use crate::maestro::MaestroVault;
    use crate::models::grpc::maestro_vault::maestro_vault_service_server::MaestroVaultService;

    pub const FILE_ID: &str = "64781cdf3236c1aef30e6188";
    pub const USER_ID: &str = "64781e773236c1aef30e6189";
    pub const DISK_ID: &str = "64781e803236c1aef30e618a";

    #[test]
    fn it_works() {
        let result = 2 + 2;

        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn sec_it_works() {
        let result = 2 + 2;

        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn upload_file_test() {
        let my_content_content = "upload_file_test";
        let my_content: Vec<u8> = my_content_content.to_string().into_bytes();
        let my_request = maestro_vault::UploadFileRequest{
            file_id: FILE_ID.to_string(),
            user_id: USER_ID.to_string(),
            disk_id: DISK_ID.to_string(),
            content: my_content};
        let vault = MaestroVault::new();
        let my_r = tonic::Request::new(my_request);
        let response = vault.upload_file(my_r).await;

        match response {
            Ok(_) => {
                // Handle the Ok variant if necessary
                println!("Value: {}", "hello");
                assert!(true)
            }
            Err(error) => {
                // Print the error message for the Err variant
                eprintln!("the error follows this message");
                eprintln!("Error: {}", error);
                assert!(false)
            }
        }

        // assert!(response.is_ok());
        // assert!(response.is_err());
    }
}