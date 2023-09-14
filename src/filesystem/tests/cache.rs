
#[cfg(test)]
mod tests {

use crate::{filesystem::{cache::CacheFS, tests::tests::{self as fs_tests, FILE_IDS, DISK_ID, FILE_CONTENTS, FILES_STORE_TYPE}, UserDiskFilesystem}, models::grpc::maestro_vault::StorageType};
use crate::maestro::i32_to_storage_type;

use std::path::Path;

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
fn _2_create_file_storage_type_test() {
    let mut file_exists: bool = false;
    let storage_type: i32 = FILES_STORE_TYPE[1].into();

    match FS.create_file(fs_tests::FILE_IDS[1], fs_tests::USER_ID, fs_tests::DISK_ID, fs_tests::FILE_CONTENTS[1].clone(), Some(i32_to_storage_type(Some(storage_type)))) {
        None => {
            let filepath = String::from("upload/") + &fs_tests::FILE_IDS[1];

            file_exists = Path::new(&filepath).exists();
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
fn _3_get_file_content_test() {
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
}

#[test]
fn _4_get_disk_files() {

    match FS.get_disk_files(fs_tests::DISK_ID) {
        Ok(files) => {
            for (file_id, file_content) in files.iter() {
                let mut file_id_was_expected = false;
                let mut file_content_was_expected = false;

                for file_id_expected in FILE_IDS {
                    if file_id == file_id_expected {
                        file_id_was_expected = true;
                    }
                }
                for file_content_expected in FILE_CONTENTS.iter() {
                    if file_content != file_content_expected {
                        file_content_was_expected = true;
                    }
                }
                if !file_id_was_expected {
                    eprintln!("\nFile id fetched with disk id is incorrect");
                    assert!(false)
                }
                if !file_content_was_expected {
                    eprintln!("\nFile content fetched with disk id  idis incorrect");
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
fn _5_get_files_disks() {
    // todo get_files_meta_info

    // todo do
    match FS.get_files_disks() {
        Ok(files_disks) => {
            // todo use several disks
            for (disk_id, disk_files) in files_disks.iter() {
                if disk_id != DISK_ID {
                    eprintln!("\nFile id fetched with disk is incorrect");
                    assert!(false)
                }
                for (file_id, file_content) in disk_files.iter() {
                    let mut file_id_was_expected = false;
                    let mut file_content_was_expected = false;

                    for file_id_expected in FILE_IDS {
                        if file_id == file_id_expected {
                            file_id_was_expected = true;
                        }
                    }
                    for file_content_expected in FILE_CONTENTS.iter() {
                        if file_content != file_content_expected {
                            file_content_was_expected = true;
                        }
                    }
                    if !file_id_was_expected {
                        eprintln!("\nFile id fetched is incorrect");
                        assert!(false)
                    }
                    if !file_content_was_expected {
                        eprintln!("\nFile content fetched is incorrect");
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
fn _6_get_user_files() {
    // todo get_files_meta_info

    // TODO get_user_files

    match FS.get_user_files(fs_tests::USER_ID) {
        Ok(files) => {
            if files.is_empty() {
                eprintln!("Did not fetch any file from user id");
                assert!(false);
            }
            for (file_id, file_content) in files.iter() {
                let mut file_id_was_expected = false;
                let mut file_content_was_expected = false;

                for file_id_expected in FILE_IDS {
                    if file_id == file_id_expected {
                        file_id_was_expected = true;
                    }
                }
                for file_content_expected in FILE_CONTENTS.iter() {
                    if file_content != file_content_expected {
                        file_content_was_expected = true;
                    }
                }
                if !file_id_was_expected {
                    eprintln!("\nFile id fetched with user id is incorrect");
                    assert!(false)
                }
                if !file_content_was_expected {
                    eprintln!("\nFile content fetched with user id  idis incorrect");
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
fn _7_get_files_store_types() {
    let file_ids = vec![FILE_IDS[0],FILE_IDS[1], FILE_IDS[2]];
    match FS.get_files_store_types(file_ids) {
        Ok(files_store_types) => {
            for (index, file_store_types) in files_store_types.iter().enumerate() {
                for store_type in file_store_types {
                    if store_type == &FILES_STORE_TYPE[index] {
                        continue;
                    } else {
                        eprintln!("\nStorage type retrived from stored file {} is not the one expected {}", store_type.as_str_name(),FILES_STORE_TYPE[index].as_str_name());
                        assert!(false)
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("\nError when storage_types disk files : {}", err);
            assert!(false)
        }
    }





}

#[test]
fn /* todo give a bigger number (execute at the end) */ _9_remove_file()  {
    FS.remove_file(fs_tests::FILE_IDS[0], fs_tests::USER_ID, fs_tests::DISK_ID);

    if Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[0])).exists() {

        eprintln!("\nCould not remove file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }
}

#[test]
fn _9_remove_file_with_storage_type()  {
    FS.remove_file(fs_tests::FILE_IDS[1], fs_tests::USER_ID, fs_tests::DISK_ID);

    if Path::new(&FS.get_default_filepath(fs_tests::FILE_IDS[1])).exists() {
        eprintln!("\nCould not remove file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }
    let store_type_path = "upload/".to_string() + fs_tests::FILE_IDS[1];
    if Path::new(&store_type_path).exists() {
        eprintln!("\nCould not remove store_type location of file {}", FS.get_default_filepath(fs_tests::FILE_IDS[0]));
        assert!(false)
    }
}

#[test]
fn /* todo give a bigger number (execute at the end) */ _9_remove_file_that_doesnt_exist()  {
    if let None = FS.remove_file("111111111111111111111111", fs_tests::USER_ID, fs_tests::DISK_ID) {
        eprintln!("\nNo error when removing inexisting file (there should have been one)");
        assert!(false)
    }

}


}