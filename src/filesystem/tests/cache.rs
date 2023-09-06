
#[cfg(test)]
mod tests {

use crate::filesystem::{cache::CacheFS, tests::tests::{self as fs_tests, FILE_IDS, DISK_ID}, UserDiskFilesystem};

use std::path::Path;
// use std::fs;
// use std::os::unix::fs as unix_fs;

// static FS: CacheFS = CacheFS{};

use lazy_static::lazy_static;

lazy_static! {
    static ref FS: CacheFS = CacheFS::new().expect("Error in cache filesystem instantiation");
}

#[test]
fn _1_create_file_test() {
    let mut file_exists: bool = false;

    match FS.create_file(fs_tests::FILE_IDS[0], fs_tests::USER_ID, fs_tests::DISK_ID, fs_tests::FILE_CONTENTS[0].clone(), None) {
        None => {
            file_exists = Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[0])).exists();
            println!("does it exist {}", file_exists); // todo remove
        }
        Some(err) => {
            eprintln!("\nCreate file error : {}", err);
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
    match FS.get_file_content(fs_tests::FILE_IDS[0]) {
        Ok(content) => {
            if content != fs_tests::FILE_CONTENTS[0] {
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
fn _3_get_disk_files() {

    match FS.get_disk_files(fs_tests::DISK_ID) {
        Ok(files) => {
            for (index, (id, content)) in files.iter().enumerate() {
                if id != FILE_IDS[index] {
                    eprintln!("\nFile id fetched with disk is incorrect");
                    assert!(false)
                }
                if content != &fs_tests::FILE_CONTENTS[index] {
                    eprintln!("\nFile content fetched with disk is incorrect");
                    assert!(false)
                }
            }
        }
        Err(err) => {
            eprintln!("\nError when fetching disk files : {}", err);
            assert!(false)
        }
    }
}

#[test]
fn _4_get_files_disk() {
    // todo get_files_meta_info

    // todo do
    match FS.get_files_disks() {
        Ok(files_disks) => {
            // todo use several disks
            for (index, (disk_id, disk_files)) in files_disks.iter().enumerate() {
                if disk_id != DISK_ID {
                    eprintln!("\nFile id fetched with disk is incorrect");
                    assert!(false)
                }
                for (index, (file_id, content)) in disk_files.iter().enumerate() {
                    if file_id != FILE_IDS[index] {
                        eprintln!("\nFile id fetched with disk is incorrect");
                        assert!(false)
                    }
                    if content != &fs_tests::FILE_CONTENTS[index] {
                        eprintln!("\nFile content fetched with disk is incorrect");
                        assert!(false)
                    }
                }

            }
        }
        Err(err) => {
            eprintln!("\nError when fetching files disks : {}", err);
            assert!(false)
        }
    }
}

#[test]
fn _5_get_user_files() {
    // todo get_files_meta_info

    // TODO get_user_files

    match FS.get_user_files(fs_tests::USER_ID) {
        Ok(files) => {
            if files.is_empty() {
                eprintln!("Did not fetch any file from user id");
                assert!(false);
            }
            for (index, (id, content)) in files.iter().enumerate() {
                if id != FILE_IDS[index] {
                    eprintln!("\nFile id fetched with user is incorrect");
                    assert!(false)
                }
                if content != &fs_tests::FILE_CONTENTS[index] {
                    eprintln!("\nFile content fetched with user is incorrect");
                    assert!(false)
                }
            }
        }
        Err(err) => {
            eprintln!("\nError when fetching disk files : {}", err);
            assert!(false)
        }
    }

}
/*
#[test]
fn /* todo give a bigger number (execute at the end) */ _9_remove_file_test()  {
    FS.remove_file(fs_tests::FILE_IDS[0], fs_tests::USER_ID, fs_tests::DISK_ID);

    if Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[0])).exists() {

        eprintln!("\nCould not remove file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }

} */


}