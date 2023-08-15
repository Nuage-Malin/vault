
mod vault;
mod cache;
mod tests;
pub mod error;

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
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: Vec<u8>, storage_type: Option<i32>) -> Option<Box<dyn Error + Send>>;

    fn remove_file(&self, file_id: &str, user_id: &str, disk_id: &str) -> Option<Box<dyn Error + Send>>; // todo remove user_id (use symlink instead of full path) or put optional

    fn set_file_content(&self, file_id: &str, content: Vec<u8>) -> Option<Box<dyn Error>>;

    // get
    fn get_file_content(&self, file_id: &str) -> Result<Vec<u8>>;
    // get_disk_files returns map with key: file_id as string, value: content as vector of u8
    fn get_disk_files(&self, disk_id: &str) -> Result<HashMap<String, Vec<u8>>>;
    // get_files_disks returns map with key: disk_id, value: map with key: file_id as string, value: content as vector of u8
    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>>;

    // get_user_files returns map with key: file_id as string, value: content as vector of u8
    fn get_user_files(&self, user_id: &str) -> Result<HashMap<String, Vec<u8>>>;

    // utils
    fn get_home_dir(&self) -> String;
    // cd_home_dir : create home dir if does not exist, then go into it
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
                        std::env::set_current_dir("..");
                    }
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }

    fn get_default_filepath(&self, file_id: &str) -> String {
        /* use std::path::PathBuf;

let mut path = PathBuf::new();
 */
        "file/".to_string() + file_id
    }
    fn get_disk_filepath(&self, disk_id: &str, file_id: &str) -> String {
        // todo add file_id to complete filepath
        /* todo
        use std::path::PathBuf;

let mut path = PathBuf::new();
 */
        let disk_path: String = String::from("disk/");

        if disk_id.is_empty() {
            disk_path
        } else {
            disk_path + disk_id + "/" + file_id
        }
    }
    fn get_user_filepath(&self, user_id: &str, file_id: &str) -> String {
        let user_path: String = String::from("user/");

        if user_id.is_empty() {
            user_path
        } else {
            user_path + user_id + "/" + file_id
        }
    }

    fn create_dir(&self, directory: &str) -> Option<Box<std::io::Error>>{
        if !Path::new(directory).exists() {
            if let Err(err) = std::fs::create_dir(directory) {
                return Some(Box::new(err));
            }
        }
        None
    }

    fn count_directories(&self, path: &str) -> usize {
        let path = Path::new(path);
        let mut count = 0;

        for component in path.components() {
            if let std::path::Component::Normal(_) = component {
                count += 1;
            }
        }

        count
    }
    // create_dir_symlink
    // If does not exist, create a directory (taking the base directory of the link parameter),
    //  then create a symbolic link into it

    fn create_symlink(&self, original: &str, link: &str) -> Option<Box<dyn Error>> {
        if let Err(err) = std::os::unix::fs::symlink(original, link) {
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
            eprintln!("\n6\n");
            return Some(Box::new(MyError::new(&(err.to_string()))));
        } else {
            return None;
        }
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
