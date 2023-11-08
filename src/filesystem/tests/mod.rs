mod vault;
mod cache;
mod helpers;

#[cfg(test)]
mod tests {

use crate::{filesystem::{self, vault::VaultFS, cache::CacheFS}, models::grpc::maestro_vault::StorageType};

use std::env;

#[test]
fn _0_select_filesystem_default() {
    env::set_var("EXEC_TYPE", "");

    match filesystem::select_filesystem() {
        Ok(fs) => {
            if let None = fs.as_any().downcast_ref::<VaultFS>() {
                // Print the error message for the Err variant
                eprintln!("\nDefault filesystem type should be vault");
                assert!(false)
            }
        }
        Err(error) => {
            eprintln!("\nError: {}", error);
            assert!(false)
        }
    }
}

#[test]
fn _0_select_filesystem_vault() {
    env::set_var("EXEC_TYPE", "vault");

    match filesystem::select_filesystem() {
        Ok(fs) => {
            if let None = fs.as_any().downcast_ref::<VaultFS>() {
                // Print the error message for the Err variant
                eprintln!("\nFilesystem type should be vault (as env var 'EXEC_TYPE' was set to 'vault')");
                assert!(false)
            }
        }
        Err(error) => {
            eprintln!("\nError: {}", error);
            assert!(false)
        }
    }
}

#[test]
fn _0_select_filesystem_cache() {
    env::set_var("EXEC_TYPE", "cache");

    match filesystem::select_filesystem() {
        Ok(fs) => {
            if let None = fs.as_any().downcast_ref::<CacheFS>() {
                // Print the error message for the Err variant
                eprintln!("\nFilesystem type should be cache (as env var 'EXEC_TYPE' was set to 'cache')");
                assert!(false)
            }
        }
        Err(error) => {
            eprintln!("\nError: {}", error);
            assert!(false)
        }
    }
}
pub const FILES_STORE_TYPE: [StorageType; 3] = [StorageType::None, StorageType::UploadQueue, StorageType::None];

pub const FILE_IDS: [&str; 3] = ["fedcba000000000000000000", "fedcba111111111111111111", "fedcba222222222222222222"];
// pub const FILE_CONTENTS: [&str; 3] = ["upload_file_test", "upload_files_test", "other_string"];

use lazy_static::lazy_static;

lazy_static! {
    pub static ref FILE_CONTENTS: [Vec<u8>; 3] = [vec!(b'h', b'e', b'l', b'l', b'o'), vec!(b'g', b'o', b'o', b'd', b'a', b'y'), vec!(b'm', b'a', b't', b'e')];
}
// pub const FILE_CONTENTS: [&[u8]; 3] = [b"upload_file_test", b"upload_files_test", b"other_string"];
pub const USER_ID: &str = "cafe00000000000000000000";
pub const DISK_ID: &str = "beef00000000000000000000";
}

