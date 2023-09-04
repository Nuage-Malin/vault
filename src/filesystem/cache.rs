use crate::filesystem;
use crate::models::grpc::maestro_vault;

use std::collections::HashMap;
use std::error::Error;
use std::any::Any;
use std::io::Write;
use std::path::Path;

use super::UserDiskFilesystem;
use super::MyError;
use super::error;

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


    fn get_file_content_from_filepath(&self, path: &str) -> Result<Vec<u8>> {
        // todo put in common methods (mod.rs)
        let ret = std::fs::read(path);

        match ret {
            Ok(content) => {
                Ok(content)
            }
            Err(err) => {
                Err(Box::new(err))
            }
        }
    }

    fn get_fileid_from_link(&self, path: &str) -> Result<String> {
/* todo todo */
        let act_path = Path::new(path);

        if let Some(id) = act_path.file_name() {
            if let Some(file_id) = id.to_str() {
                return Ok(String::from(file_id));
            }
        }
        return Err(Box::new(MyError::new("Could not get original filepath from symbolic link path")));
    }

}

impl filesystem::UserDiskFilesystem for CacheFS {
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: Vec<u8>, storage_type: Option<i32>) -> Option<Box<dyn Error + Send>>{
        if !self.is_cur_dir_home_dir() { // todo useless if we check it in class instantiation (function `new`) ?
            return Some(Box::new(MyError::new("Current directory should be home directory of the filesystem")));
        }
        let filepath = self.get_default_filepath(file_id);

        { // create and write file
            match std::fs::File::create(&filepath) {
                Ok(mut file) => {
                    match file.write_all(&content) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("1");
                            return Some(Box::new(MyError::new(&(err.to_string()))));
                        }
                    }
                }
                Err(err) => {
                    eprintln!("2");
                    return Some(Box::new(MyError::new(&(err.to_string()))));
                }
            }
        }
        if let Some(store) = storage_type {
            if let Some(path_start) = self.store_paths.get(&store) {
                let store_filepath = path_start.to_string() + &filepath; // todo make that a method and put the method into the map

                if let Some(err) = self.create_symlink( &filepath, &store_filepath) {
                    eprintln!("3");
                    return Some(Box::new(MyError::new(&(err.to_string()))));
                }
            }
        }
        // todo create directory for user and disk (each id has to have a directory)
        if let Some(err) = self.create_dir(&(self.get_user_filepath(user_id, ""))) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_symlink( &filepath, &(self.get_user_filepath(user_id, file_id))) {
            eprintln!("4 {}, {}", filepath, &(self.get_user_filepath(user_id, file_id))); // todo remove

            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_dir(&(self.get_disk_filepath(disk_id, ""))) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_symlink( &filepath, &(self.get_disk_filepath(disk_id, file_id))) {
            eprintln!("5 {}, {}", filepath, &(self.get_disk_filepath(disk_id, file_id))); // todo remove

            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        None
    }

    fn remove_file(&self, file_id: &str, user_id: &str, disk_id: &str) -> Option<Box<dyn Error + Send>>{
        if let Err(err) = std::fs::remove_file(self.get_default_filepath(&file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_disk_filepath(&disk_id, &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_user_filepath(&user_id, &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        None
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

    fn get_file_content(&self, file_id: &str) -> Result<Vec<u8>> {
        let path = self.get_default_filepath(file_id);

        self.get_file_content_from_filepath(&path)
    }

    // get_disk_files returns map with key: file_id as string, value: content as vector of u8
    fn get_disk_files(&self, disk_id: &str) -> Result<HashMap<String, Vec<u8>>>{
        let mut files: HashMap<String, Vec<u8>> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.get_disk_filepath(disk_id, "")) {
            for entry in entries {
                if let Ok(file) = entry {
                    if let Some(filepath) = file.path().to_str() {

                        // println!("file path : {}", basename(filepath));
                        match self.get_fileid_from_link(filepath) {
                            Ok (fileid) => {
                                match std::fs::read(file.path()) {
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

    // get_files_disks returns map with key: disk_id, value: map with key: file_id as string, value: content as vector of u8
    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>>{
        // todo now
        // todo replicate for vault fs
        let files_disks: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
        // iterate through dirs in 'disks' dir, then iterate through  to fill up the map
        let disk_str_dirpath = self.get_disk_filepath("", "");
        let disk_dirpath = std::path::Path::new(&disk_str_dirpath);

        match std::fs::read_dir(disk_dirpath) {
            Ok(dir) => {
                for potential_entry in dir {
                    match potential_entry {
                        Ok(entry) => {
                            if let Some(filename) = entry.file_name().as_os_str().to_str() {
                                eprintln!("dir entry : {}", filename);
                            }
                        }
                        Err(_) => {}
                    }


                }
            }
            Err(err) => {
                return Err(Box::new(MyError::new(&(err.to_string()))));
            }
        }



        return Ok(files_disks);
    }

    // get_user_files returns map with key: file_id as string, value: content as vector of u8
    fn get_user_files(&self, _user_id: &str) -> Result<HashMap<String, Vec<u8>>>{
        // todo now
        // todo replicate for vault fs
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