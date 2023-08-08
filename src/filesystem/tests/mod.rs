mod vault;
mod cache;

#[cfg(test)]
mod tests {

use crate::filesystem::{self, vault::VaultFS, cache::CacheFS};

use std::env;

#[test]
fn _0_select_filesystem_default() {
    env::set_var("EXEC_TYPE", "");

    let fs: Box<dyn filesystem::UserDiskFilesystem> = filesystem::select_filesystem();

    if let None = fs.as_any().downcast_ref::<VaultFS>() {
        // Print the error message for the Err variant
        eprintln!("\nDefault filesystem type should be vault");
        assert!(false)
    }
}

#[test]
fn _0_select_filesystem_vault() {
    env::set_var("EXEC_TYPE", "vault");

    let fs: Box<dyn filesystem::UserDiskFilesystem> = filesystem::select_filesystem();

    if let None = fs.as_any().downcast_ref::<VaultFS>() {
        // Print the error message for the Err variant
        eprintln!("\nFilesystem type should be vault (as env var 'EXEC_TYPE' was set to 'vault')");
        assert!(false)
    }
}

#[test]
fn _0_select_filesystem_cache() {
    env::set_var("EXEC_TYPE", "cache");

    let fs: Box<dyn filesystem::UserDiskFilesystem> = filesystem::select_filesystem();

    if let None = fs.as_any().downcast_ref::<CacheFS>() {
        // Print the error message for the Err variant
        eprintln!("\nFilesystem type should be cache (as env var 'EXEC_TYPE' was set to 'cache')");
        assert!(false)
    }
}
pub const FILE_IDS: [&str; 3] = ["53452cdf4734c1aef76e5254", "14356e97f0c3a443451351b0", "25467b1bf0c3a254678257b1"];
pub const _FILE_CONTENTS: [&str; 3] = ["upload_file_test", "upload_files_test", "second_string"];
pub const USER_ID: &str = "14361e624622c1aef30e5425";
pub const DISK_ID: &str = "14361e456568c1aef56e765a";
}

