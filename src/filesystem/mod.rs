
mod vault;
mod cache;
mod tests;
mod encryption;
pub mod error;

use crate::models::grpc::maestro_vault::StorageType;
use crate::my_eprintln;

use error::MyError;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::any::Any;
use std::path::Path;
use std::io::{Read, Write};

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

pub trait UserDiskFilesystem: Send + Sync {
    // Associated function signature; `Self` refers to the implementor type.
    // fn new(&self) -> Self;

    // todo replicate methods to handle multiple files at once


    /// Create ///

    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: &[u8], storage_type: Option<StorageType>) -> Option<Box<dyn Error + Send>>;

    fn create_dir(&self, directory: &str) -> Option<Box<dyn Error + Send>>{
        if !Path::new(directory).exists() {
            if let Err(err) = std::fs::create_dir(directory) {
                return Some(Box::new(MyError::new(&err.to_string())));
            }
        }
        None
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


    /// Set ///
    ///
    fn set_file_content_from_id(&self, file_id: &str, content: &[u8]) -> Option<Box<dyn Error + Send>>{
        let filepath = self.get_default_filepath(file_id);

        return self.set_file_content_from_filepath(&filepath, content);
    }

    fn set_file_content_from_filepath(&self, filepath: &str, content: &[u8]) -> Option<Box<dyn Error + Send>> {
        match std::fs::File::open(filepath) {
            Ok(file) => {
                return self.set_file_content(&file, content);
            }
            Err(r) => {
                Some(Box::new(MyError::new(&r.to_string())))
            }
        }
    }

    fn set_file_content(&self, file: &std::fs::File, content: &[u8]) -> Option<Box<dyn Error + Send>> {
        match encryption::FileEncryption::encrypt(content, /* TODO key */) {
            Ok(encrypted_content) => {
                match file.write_all(&encrypted_content) {
                    Ok(_) => {None}
                    Err(err) => {
                        return Some(Box::new(MyError::new(&(err.to_string()))));
                    }
                }
            }
            Err(r) => {
                Some(Box::new(MyError::new(&r.to_string())))
            }
        }
    }

    /// Remove ///

    fn remove_file(&self, file_id: &str) -> Option<Box<dyn Error + Send>>; // todo remove user_id (use symlink instead of full path) or put optional

    /// remove directory and all subfiles without warning, use carefully !
    fn remove_directory(&self, dirpath: &str) -> Option<Box<dyn Error + Send>>
    {
        if let Err(err) = std::fs::remove_dir_all(dirpath) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        return None
    }

    /// remove user files and directories
    fn remove_user(&self, user_id: &str) -> Option<Box<dyn Error + Send>>
    {
        match self.get_user_files(user_id)/* todo get_user_file_ids instead */ {
            Ok(files) => {
                for file_id in files.keys() {
                    self.remove_file(file_id);
                }
            }
            Err(err) => {
                return Some(err)
            }
        }
        self.remove_directory(&self.get_user_filepath(user_id, ""));
        None
    }


    /// Get ///
    ///
    fn get_file_content_from_id(&self, file_id: &str) -> Result<Vec<u8>> {
        let filepath = self.get_default_filepath(file_id);

        return self.get_file_content_from_filepath(&filepath);
    }
    fn get_file_content_from_filepath(&self, filepath: &str) -> Result<Vec<u8>> {
        // todo use
        let res = std::fs::File::open(filepath);

        match res {
            Ok(file) => {
                return self.get_file_content(&file);
            }
            Err(err) => {
                Err(Box::new(err))
            }
        }
    }

    fn get_file_content(&self, file: &std::fs::File) -> Result<Vec<u8>>{
        let mut buf_encrypted_content: Vec<u8>;

        match file.read_to_end(&mut buf_encrypted_content) {
            Ok(read_size) => {
                match encryption::FileEncryption::decrypt(&buf_encrypted_content, /* TODO key */) {
                    Ok(decrypted_content) => {
                        Ok(decrypted_content)
                    }
                    Err(r) => {
                        Err(Box::new(MyError::new(&r.to_string())))
                    }
                }
            }
            Err(r) => {
                Err(Box::new(MyError::new(&r.to_string())))
            }
        }
    }

    /// get_disk_files returns map with key: file_id as string, value: content as vector of u8
    fn get_disk_files(&self, disk_id: &str) -> Result<HashMap<String, Vec<u8>>> {
        let mut files: HashMap<String, Vec<u8>> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.get_disk_filepath(disk_id, "")) {
            for entry in entries {
                if let Ok(file_entry) = entry {
                    if let Some(filepath) = file_entry.path().to_str() {

                        // println!("file path : {}", basename(filepath));
                        match self.get_fileid_from_path(filepath) {
                            Ok (fileid) => {
                                match std::fs::read(file_entry.path()) {
                                    Ok(content) => {
                                        files.insert(fileid /* filename of link */, content);
                                    }
                                    Err(err) => {
                                        return Err(Box::new(err));
                                    }
                                }
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    }
                    // todo test
                }
            }
        }
        return Ok(files);
    }

    /// get_files_disks returns map with key: disk_id, value: map with key: file_id as string, value: content as vector of u8
    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>>;

    /// get_user_files returns map with key: file_id as string, value: content as vector of u8
    // todo create a get_user_file_ids
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
        let link_path = self.get_default_dirpath(file_id) + "/user";

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

                /* if !path.exists() {
                    return Err(Box::new(MyError::new(&format!("Line {} in {} : disk path doesn't exist", line!(), file!()))));
                } */
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

                /* if !path.exists() {
                    return Err(Box::new(MyError::new(&format!("Line {} in {} : user path doesn't exist", line!(), file!()))));
                } */
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

    fn get_fileid_from_path(&self, path: &str) -> Result<String> {
        let act_path = Path::new(path);

        if let Some(id) = act_path.file_name() {
            if let Some(file_id) = id.to_str() {
                return Ok(String::from(file_id));
            }
        }
        return Err(Box::new(MyError::new("Could not get original filepath from symbolic link path")));
    }

    /// return hashmap with file_id as key, store_types as value
    fn get_all_files_store_types(&self) -> Result<HashMap<String, Vec<StorageType>>>;

    /// return store_types
    fn get_file_store_types(&self, file_id: &str) -> Result<Vec<StorageType>>;

    /// return Vector with order corresponding to file_ids given as paramater, store_types
    fn get_files_store_types(&self, file_id: Vec<&str>) -> Result<Vec<Vec<StorageType>>>;

    /// return vector with file_ids
    fn get_store_type_files(&self, store_type: StorageType) -> Result<Vec<String>>;

    fn get_home_dir(&self) -> String;


    /// Utils ///

    /// cd_home_dir : create home dir if does not exist, then go into it
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
                                        my_eprintln!("{}", err);
                                        panic!();
                                    }
                                }
                            }
                        }

                        res = std::env::set_current_dir(&home_dir);
                        match res {
                            Ok(_) => {}
                            Err(err) => {
                                my_eprintln!("{}", err);
                                panic!();
                            }
                        }
                    }
                }
            }
            Err(err) => {
                my_eprintln!("{}", err);
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
                                my_eprintln!("Could not set current dir to parent : {}", err.to_string());
                            }
                        }
                    }
                }
            }
            Err(err) => {
                my_eprintln!("Could not get current dir : {}", err.to_string());
            }
        }
    }

    /// Optional parameter indicates if the path includes the filename at it's end
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

/// Select filesystem : If env var cache then cache filesystem, else vault
pub fn select_filesystem() -> Result<Box<dyn UserDiskFilesystem>> {

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
        "vault-cache" => {
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
