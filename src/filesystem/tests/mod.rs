mod vault;
mod cache;

#[cfg(test)]
mod tests {

use crate::filesystem::{self, vault::VaultFS, cache::CacheFS};

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
pub const FILE_IDS: [&str; 3] = ["53452cdf4734c1aef76e5254", "14356e97f0c3a443451351b0", "25467b1bf0c3a254678257b1"];
// pub const FILE_CONTENTS: [&str; 3] = ["upload_file_test", "upload_files_test", "other_string"];

use lazy_static::lazy_static;

lazy_static! {
    pub static ref FILE_CONTENTS: [Vec<u8>; 3] = [vec!(b'h', b'e', b'l', b'l', b'o'), vec!(b'g', b'o', b'o', b'd', b'a', b'y'), vec!(b'm', b'a', b't', b'e')];
}
// pub const FILE_CONTENTS: [&[u8]; 3] = [b"upload_file_test", b"upload_files_test", b"other_string"];
pub const USER_ID: &str = "14361e624622c1aef30e5425";
pub const DISK_ID: &str = "14361e456568c1aef56e765a";
}

