
#[cfg(test)]
mod tests {

use crate::filesystem::{vault::VaultFS, tests::tests as fs_tests, UserDiskFilesystem};

use std::path::Path;
// use std::fs;
// use std::os::unix::fs as unix_fs;

// static FS: VaultFS = VaultFS{};

use lazy_static::lazy_static;

lazy_static! {
    static ref FS: VaultFS = VaultFS::new();
}

#[test]
fn _1_create_file_test() {
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
fn _2_get_file_content_test() {
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
fn /* todo give a bigger number (execute at the end) */ _9_remove_file_test()  {
    FS.remove_file(fs_tests::FILE_IDS[0], fs_tests::USER_ID, fs_tests::DISK_ID);

    let file_exists = Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[0])).exists();

    if file_exists {

        eprintln!("\nCould not remove file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }

}


}