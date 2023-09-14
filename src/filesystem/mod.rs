
mod vault;
mod cache;
mod tests;
pub mod error;

use crate::models::grpc::maestro_vault::{self, StorageType};

use error::MyError;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::any::Any;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

pub trait UserDiskFilesystem: Send + Sync {
    // Associated function signature; `Self` refers to the implementor type.
    // fn new(&self) -> Self;

    // todo replicate methods to handle multiple files at once

    // set
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: Vec<u8>, storage_type: Option<StorageType>) -> Option<Box<dyn Error + Send>>;

    fn remove_file(&self, file_id: &str, user_id: &str, disk_id: &str) -> Option<Box<dyn Error + Send>>; // todo remove user_id (use symlink instead of full path) or put optional

    fn set_file_content(&self, file_id: &str, content: Vec<u8>) -> Option<Box<dyn Error>>;

    // get
    fn get_file_content(&self, file_id: &str) -> Result<Vec<u8>>;

    // get_disk_files returns map with key: file_id as string, value: content as vector of u8
    fn get_disk_files(&self, disk_id: &str) -> Result<HashMap<String, Vec<u8>>>;

    // get_files_disks returns map with key: disk_id, value: map with key: file_id as string, value: content as vector of u8
    // todo how to know which user the files belong to ?
    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>>;

    // get_user_files returns map with key: file_id as string, value: content as vector of u8
    fn get_user_files(&self, user_id: &str) -> Result<HashMap<String, Vec<u8>>>;

    fn get_diskpath_from_file(&self, file_id: &str) -> Result<String> {
        let link_path = self.get_default_dirpath(file_id) + "/disk";

        match std::fs::read_link(link_path) {
            Ok(target_path) => {
                if let Some(target_str) = target_path.to_str() {
                    // println!("Symbolic Link '{}' points to: {}", link_path, target_str);
                    return Ok(target_str.to_string());
                }
            }
            Err(err) => {
                return Err(Box::new(MyError::new(&err.to_string())));
            }
        }
        return Err(Box::new(MyError::new(&format!("Line {} in {} : Could not diskpath from file id", line!(), file!()))));
    }

    fn get_userpath_from_file(&self, file_id: &str) -> Result<String> {
        let link_path = self.get_default_dirpath(file_id) + "/disk";

        match std::fs::read_link(link_path) {
            Ok(target_path) => {
                if let Some(target_str) = target_path.to_str() {
                    // println!("Symbolic Link '{}' points to: {}", link_path, target_str);
                    return Ok(target_str.to_string());
                }
            }
            Err(err) => {
                return Err(Box::new(MyError::new(&err.to_string())));
            }
        }
        return Err(Box::new(MyError::new(&format!("Line {} in {} : Could not diskpath from file id", line!(), file!()))));
    }

    fn get_file_disk(&self, file_id: &str) -> Result<String> {
        match self.get_diskpath_from_file(file_id) {
            Ok(diskpath) => {
                let path = Path::new(&diskpath); // get basename which is the directory named with the disk id

                if !path.exists() {
                    return Err(Box::new(MyError::new(&format!("Line {} in {} : disk path doesn't exist", line!(), file!()))));
                }
                if let Some(filename) = path.file_name() {
                    if let Some(filename) = filename.to_str() {
                        return Ok(filename.to_string());
                    }
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
        return Err(Box::new(MyError::new(&format!("Line {} in {} : disk path doesn't exist", line!(), file!()))));
    }

    fn get_file_user(&self, file_id: &str) -> Result<String> {
        match self.get_userpath_from_file(file_id) {
            Ok(userpath) => {
                let path = Path::new(&userpath); // get basename which is the directory named with the user id

                if !path.exists() {
                    return Err(Box::new(MyError::new(&format!("Line {} in {} : user path doesn't exist", line!(), file!()))));
                }
                if let Some(filename) = path.file_name() {
                    if let Some(filename) = filename.to_str() {
                        return Ok(filename.to_string());
                    }
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
        return Err(Box::new(MyError::new(&format!("Line {} in {} : user path doesn't exist", line!(), file!()))));
    }

    /* return hashmap with file_id as key, store_types as value */
    fn get_all_files_store_types(&self) -> Result<HashMap<String, Vec<maestro_vault::StorageType>>>;

    /* return store_types */
    fn get_file_store_types(&self, file_id: &str) -> Result<Vec<maestro_vault::StorageType>>;

    /* return Vector with order corresponding to file_ids given as paramater, store_types */
    fn get_files_store_types(&self, file_id: Vec<&str>) -> Result<Vec<Vec<maestro_vault::StorageType>>>;

    /* return vector with file_ids  */
    fn get_store_type_files(&self, store_type: StorageType) -> Result<Vec<String>>;

    // utils
    fn get_home_dir(&self) -> String;

    // cUploadFileStatusd_home_dir : create home dir if does not exist, then go into it
    fn cd_home_dir(&self) -> Option<Box<dyn Error>> {
        let home_dir = self.get_home_dir();

        match std::env::current_dir() {
            Ok(cur) => {
                if let Some(act_dir) = cur.file_name().unwrap().to_str() {
                    if act_dir != home_dir {
                        let mut res = std::fs::create_dir(&home_dir); // create dir if doesn't exist

                        match res {
                            Ok(_) => {}
                            Err(err) => {
                                match err.kind() {
                                    std::io::ErrorKind::AlreadyExists => {}
                                    _ => {
                                        eprintln!("{}", err);
                                        panic!();
                                    }
                                }
                            }
                        }
                        res = std::env::set_current_dir(&home_dir);
                        match res {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("{}", err);
                                panic!();
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                panic!()
            }
        }
        None
    }

    fn cd_home_dir_parent(&self) {
        let home_dir = self.get_home_dir();

        match std::env::current_dir() {
            Ok(cur) => {
                if let Some(act_dir) = cur.file_name().unwrap().to_str() {
                    if act_dir == home_dir {
                        match std::env::set_current_dir("..") {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("Line {} in {} : Could not set current dir to parent : {}", line!(), file!(), err.to_string());
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Line {} in {} : Could not get current dir : {}", line!(), file!(), err.to_string());
            }
        }
    }

    fn get_default_dirpath(&self, file_id: &str) -> String {
/*         if !Path::new(&dirpath).exists() {
            // If dir path doesn't exist, create one
            if let Some(err) = self.create_dir(&dirpath) {
                return Err(err);
            }
        } */
        "file/".to_string() + file_id
    }

    fn get_default_filepath(&self, file_id: &str) -> String {
        self.get_default_dirpath(file_id) + "/file"
    }

    fn get_disk_filepath(&self, disk_id: &str, file_id: &str /* todo Option<> ? */) -> String {
        let disk_path: String = String::from("disk/");

        if disk_id.is_empty() {
            disk_path
        } else {
            disk_path + disk_id + "/" + file_id
        }
    }

    fn get_user_filepath(&self, user_id: &str, file_id: &str /* todo Option<> ? */) -> String {
        let user_path: String = String::from("user/");

        if user_id.is_empty() {
            user_path
        } else {
            user_path + user_id + "/" + file_id
        }
    }

    fn create_dir(&self, directory: &str) -> Option<Box<dyn Error + Send>>{
        if !Path::new(directory).exists() {
            if let Err(err) = std::fs::create_dir(directory) {
                return Some(Box::new(MyError::new(&err.to_string())));
            }
        }
        None
    }

    /*
    Optional parameter indicates if the path includes the filename at it's end
     */
    /* todo put in a different mod : helpers */
    fn count_directories(&self, path: &str, has_filename: Option<bool>) -> usize {
        let path = Path::new(path);
        let mut count = 0;

        for component in path.components() {
            if let std::path::Component::Normal(_) = component {
                count += 1;
            }
        }
        if count > 0 && (path.is_file() || has_filename.unwrap_or(false)) {
            count -= 1;
        }
        count
    }

    fn create_hardlink(&self, initial: &str, link: &str) -> Option<Box<dyn Error + Send>> {
        if let Err(err) = std::fs::hard_link(initial, link) {
            if let Some(error_kind) = err // Obtain the error kind
                .source()
                .and_then(|err| err.downcast_ref::<std::io::Error>())
                .map(|io_error| io_error.kind())
            {
                match error_kind {
                    std::io::ErrorKind::AlreadyExists => {
                        return None // if error is 'already exists', ignore it
                    }
                    _ => {}
                }
            } else if let Some(kind) = err.raw_os_error() {
                if kind == 17 /* os error : file exists  */{
                    return None;
                }
            }
            return Some(Box::new(MyError::new(&(err.to_string()))));
        } else {
            return None;
        }
    }

    fn create_symlink(&self, original: &str, link: &str, link_has_filename: Option<bool>) -> Option<Box<dyn Error + Send>>{
        let link_dir_num = self.count_directories(link, link_has_filename);
        let act_original = "../".repeat(link_dir_num) + original; /* todo test */

        if let Err(err) = std::os::unix::fs::symlink(act_original, link) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        None
    }

    fn is_cur_dir_home_dir(&self) -> bool {
        match std::env::current_dir() {
            Ok(cur) => {
                if let Some(act_dir) = cur.to_str() {
                    let cur_path = Path::new(act_dir);

                    if let Some(basename) = cur_path.file_name() {
                        if let Some(basename) = basename.to_str() {
                            if basename == self.get_home_dir() {
                                return true;
                            }
                        }
                    }
                    // println!("pwd : {}", act_dir); // todo remove
                }
            }
            _ => {
                return false;
            }
        }
        return false;
    }

///
    fn as_any(&self) -> &dyn Any;

}

impl std::fmt::Debug for dyn UserDiskFilesystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserDiskFilesystem")
            .finish()
    }
}

impl Default for Box<dyn UserDiskFilesystem> {
    fn default() -> Self {
        Box::new(vault::VaultFS{})
    }
}

pub fn select_filesystem() -> Result<Box<dyn UserDiskFilesystem>> {
    // if // env var cache then cache filesystem, else vault
    let exec_type = env::var("EXEC_TYPE").expect("EXEC_TYPE not set.");

    match exec_type.as_str() {
        "vault" => {
            match vault::VaultFS::new() {
                Ok(vault) => {
                    return Ok(Box::new(vault));
                }
                Err(err) => {
                    return Err(err);
                }
            }

        }
        "cache" => {
            match cache::CacheFS::new() {
                Ok(cache) => {
                    return Ok(Box::new(cache));
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        _ => {
            match vault::VaultFS::new() {
                Ok(vault) => {
                    return Ok(Box::new(vault));
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }
}
