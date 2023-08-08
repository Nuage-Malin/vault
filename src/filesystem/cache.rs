use crate::filesystem;
use crate::models::grpc::maestro_vault;

use std::collections::HashMap;
use std::error::Error;
use std::any::Any;
use std::io::Write;

use super::UserDiskFilesystem;
use super::MyError;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct CacheFS {
    store_paths: HashMap<i32, String>
}

impl CacheFS {

    pub fn new() -> Result<Self> {
        // todo create and go into cache_fs dir
        let mut map: HashMap<i32, String> = HashMap::new();
        map.insert(maestro_vault::StorageType::UploadQueue.into(), "upload/".to_string());
        map.insert(maestro_vault::StorageType::DownloadQueue.into(), "download/".to_string());
        map.insert(maestro_vault::StorageType::RemoveQueue.into(), "remove/".to_string());
        let cache_fs = CacheFS{store_paths: map};

        cache_fs.cd_home_dir();
        if let Some(err) = cache_fs.create_dir(&cache_fs.get_default_filepath("")) {
            return Err(err);
        }
        if let Some(err) = cache_fs.create_dir(&cache_fs.get_user_filepath("", "")) {
            return Err(err);
        }
        if let Some(err) = cache_fs.create_dir(&cache_fs.get_disk_filepath("", "")) {
            return Err(err);
        }

        return Ok(cache_fs);
    }

}

impl filesystem::UserDiskFilesystem for CacheFS {
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: Vec<u8>, storage_type: Option<i32>) -> Option<Box<dyn Error + Send>>{
        let filepath = self.get_default_filepath(file_id);

        if !self.is_cur_dir_home_dir() { // todo useless if we check it in class instantiation (function `new`) ?
            return Some(Box::new(MyError::new("Current directory should be home directory of the filesystem")));
        }
        { // create and write file
            let ret = std::fs::File::create(&filepath);

            match ret {
                Ok(mut file) => {
                    let res = file.write_all(&content);
                    match res {
                        Ok(_) => {}
                        Err(err) => {
                            return Some(Box::new(MyError::new(&(err.to_string()))));
                        }
                    }
                }
                Err(err) => {
                    return Some(Box::new(MyError::new(&(err.to_string()))));
                }
            }
        }
        if let Some(store) = storage_type {
            if let Some(path_start) = self.store_paths.get(&store) {
                let store_filepath = path_start.to_string() + &filepath; // todo make that a method and put the method into the map

                if let Some(err) = self.create_symlink( &filepath, &store_filepath) {
                    return Some(Box::new(MyError::new(&(err.to_string()))));
                }
            }
        }
        if let Some(err) = self.create_symlink( &filepath, &self.get_user_filepath(user_id, file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_symlink( &filepath, &(self.get_disk_filepath(disk_id, file_id) + "/" + file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        None
    }

    fn remove_file(&self, file_id: &str, user_id: &str, disk_id: &str) -> Option<Box<dyn Error + Send>>{
        let ret = std::fs::remove_file(self.get_default_filepath(file_id));

        match ret {
            Ok(_) => {None}
            Err(err) => {
                return Some(Box::new(MyError::new(&(err.to_string()))));
            }
        }
    } // todo remove user_id (use symlink instead of full path) or put optional

    fn set_file_content(&self, file_id: &str, content: Vec<u8>) -> Option<Box<dyn Error>>{
        let filepath = self.get_default_filepath(file_id);
        let ret = std::fs::write(&filepath, &content);

        match ret {
            Ok(_) => {None}
            Err(err) => {
                return Some(Box::new(MyError::new(&(err.to_string()))));
            }
        }
    }

    // get
    fn get_file_content(&self, file_id: &str) -> Result<Vec<u8>>{
        let ret = std::fs::read(self.get_default_filepath(file_id));

        match ret {
            Ok(content) => {
                Ok(content)
            }
            Err(err) => {
                Err(Box::new(err))
            }
        }
    }
    // get_disk_files returns map with key: file_id as string, value: content as vector of u8
    fn get_disk_files(&self, disk_id: &str) -> Result<HashMap<String, Vec<u8>>>{
        // std::fs::read_link(path)
        let files: HashMap<String, Vec<u8>> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.get_disk_filepath(disk_id, "")) {
            for entry in entries {
                if let Ok(file) = entry {
                    let link = std::fs::read_link(file.path());

                    print!("{}", link.unwrap().display());
                    // todo finish
                }

            }
        }

        return Ok(files);
    }
    // get_files_disks returns map with key: disk_id, value: map with key: file_id as string, value: content as vector of u8
    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>>{
        let files_disks: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();

        return Ok(files_disks);
    }

    // get_user_files returns map with key: file_id as string, value: content as vector of u8
    fn get_user_files(&self, _user_id: &str) -> Result<HashMap<String, Vec<u8>>>{
        let files: HashMap<String, Vec<u8>> = HashMap::new();

        return Ok(files);
    }
    fn get_home_dir(&self) -> String {
        String::from("cache_fs")
    }

///
    fn as_any(&self) -> &dyn Any {
        self
    }

}


impl Drop for CacheFS {
    fn drop(&mut self) {
        self.cd_home_dir_parent();
    }
}