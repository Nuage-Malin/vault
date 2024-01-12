
#[cfg(test)]
mod tests {

use crate::filesystem::{vault::VaultFS, tests::tests as fs_tests, UserDiskFilesystem};
use crate::my_eprintln;

use std::path::Path;

use lazy_static::lazy_static;

lazy_static! {
    static ref FS: VaultFS = VaultFS::new().expect("Error in vault filesystem instantiation");
}

#[test]
fn _01_create_file_test() {
    let content = vec![1, 2, 3, 4, 5];
    let res = FS.create_file(fs_tests::FILE_IDS[0], fs_tests::USER_ID, fs_tests::DISK_ID, content, None);
    let mut file_exists: bool = false;

    match res {
        None => {
            file_exists = Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[0])).exists();
        }
        Some(err) => {
            eprintln!("{}", err);
        }
    }

    if !file_exists {

        eprintln!("\nCould not create file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }
    // todo check symbolic links
}

#[test]
fn _02_get_file_content_test() {
    let expected_content: Vec<u8> = vec![1, 2, 3, 4, 5];
    let file_content = FS.get_file_content(fs_tests::FILE_IDS[0]);

    match file_content {
        Ok(content) => {
            if content != expected_content {
                eprintln!("\nContent fetched from file is different than when created");
                assert!(false)
            }
        }
        Err(err) => {
            eprintln!("\nError when fetching file content : {}", err);
            assert!(false)
        }
    }

    // todo check symbolic links
}

#[test]
fn /* todo give a bigger number (execute at the end) */ _09_remove_file_test()  {
    FS.remove_file(fs_tests::FILE_IDS[0]);

    let file_exists = Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[0])).exists();

    if file_exists {

        eprintln!("\nCould not remove file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }
}

#[test]
fn _10_remove_non_existing_user() {
    let non_existing_user_id = "000000000000000000000000";
    let user_filepath = FS.get_user_filepath(non_existing_user_id, "");

    if Path::new(&user_filepath).exists() {
        my_eprintln!("\nDirectory '{}' makes test `{}` impossible", non_existing_user_id, std::stringify!(_10_remove_non_existing_user));
        assert!(false)
    }
    if let None = FS.remove_user(non_existing_user_id) {
        my_eprintln!("\nNo error when removing non existing user (there should have been one)");
        assert!(false)
    }
}

#[test]
fn _11_remove_user() {
    if let Some(err) = FS.remove_user(fs_tests::USER_ID) {
        my_eprintln!("\nError when removing user : {}", err.to_string());
        assert!(false)
    }
    let user_filepath = FS.get_user_filepath(fs_tests::USER_ID, "");

    if Path::new(&user_filepath).exists() {
        my_eprintln!("\nUser directory still exists (it should've been removed) : {}", &user_filepath);
        assert!(false)
    }

    // todo check if user directory is still there
}

#[test]
fn _12_remove_already_removed_user_test() {
    if let None = FS.remove_user(fs_tests::USER_ID) {
        my_eprintln!("\nErroneously removed already removed user");
        assert!(false)
    }
}
}
