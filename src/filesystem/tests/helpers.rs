
#[cfg(test)]
mod tests {

use crate::{filesystem::{cache::CacheFS, tests::tests::{self as fs_tests, FILE_IDS, DISK_ID, FILE_CONTENTS, FILES_STORE_TYPE}, UserDiskFilesystem}, models::grpc::maestro_vault::StorageType};

use lazy_static::lazy_static;

lazy_static! {
    // static ref FS: CacheFS = CacheFS::new().expect("Error in cache filesystem instantiation");
    static ref FS: CacheFS = CacheFS::new().expect("Error in cache filesystem instantiation");
}

// test name convention :
// _{number defining the execution order}_{test name, often the name of the function/method tested}_{test number, if several tests for the same function/method}

#[test]
fn _0_count_directories_0() {
    let path = "path/to/file";
    let expected_dir_num = 2;
    let dir_num = FS.count_directories(path, Some(true));

    if expected_dir_num != dir_num {
        eprintln!("Did not count directories correctly, expected {} got {}", expected_dir_num, dir_num);
        assert!(expected_dir_num == dir_num);
    }
}

#[test]
fn _0_count_directories_1() {
    let path = "path/to/dir";
    let expected_dir_num = 3;
    let dir_num = FS.count_directories(path, Some(false));

    if expected_dir_num != dir_num {
        eprintln!("Did not count directories correctly, expected {} got {}", expected_dir_num, dir_num);
        assert!(expected_dir_num == dir_num);
    }
}

#[test]
fn _0_count_directories_2() {
    let path = "dirpath";
    let expected_dir_num = 1;
    let dir_num = FS.count_directories(path, Some(false));

    if expected_dir_num != dir_num {
        eprintln!("Did not count directories correctly, expected {} got {}", expected_dir_num, dir_num);
        assert!(expected_dir_num == dir_num);
    }
}

#[test]
fn _0_count_directories_3() {
    let path = "filepath";
    let expected_dir_num = 0;
    let dir_num = FS.count_directories(path, Some(true));

    if expected_dir_num != dir_num {
        eprintln!("Did not count directories correctly, expected {} got {}", expected_dir_num, dir_num);
        assert!(expected_dir_num == dir_num);
    }
}
}