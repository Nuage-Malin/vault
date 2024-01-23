use crate::filesystem;
use crate::filesystem::UserDiskFilesystem;
use crate::my_eprintln;

use crate::models::grpc::maestro_vault::{self, StorageType};

use std::collections::HashMap;
use std::error::Error;
use std::any::Any;
use std::path::Path;

use super::MyError;
// use super::error;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct CacheFS {
    store_paths: HashMap<maestro_vault::StorageType, String>
}

impl CacheFS {

    pub fn new() -> Result<Self> {
        let mut map: HashMap<maestro_vault::StorageType, String> = HashMap::new();
        map.insert(maestro_vault::StorageType::UploadQueue, "upload".to_string());
        map.insert(maestro_vault::StorageType::DownloadQueue, "download".to_string());
        let cache_fs = CacheFS{store_paths: map};

        cache_fs.cd_home_dir();
        if let Some(err) = cache_fs.create_dir(&cache_fs.get_default_dirpath("")) {
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

    fn get_fileid_from_path(&self, path: &str) -> Result<String> {
        let act_path = Path::new(path);

        if let Some(id) = act_path.file_name() {
            if let Some(file_id) = id.to_str() {
                return Ok(String::from(file_id));
            }
        }
        return Err(Box::new(MyError::new("Could not get original filepath from symbolic link path")));
    }

    fn get_store_type_dir(&self, store: &StorageType) -> Option<String> {
        if store == &StorageType::None {
            return None;
        }
        if let Some(store_path) = self.store_paths.get(&store) {
            return Some(store_path.to_string());
        }
        return None;
    }

    fn get_store_type_dirs_from_file(&self, file_id: &str) -> Option<Vec<String>> {
        let mut store_type_dirs: Vec<String> = vec![];
        let dirpath = self.get_default_dirpath(file_id);

        for (_store_type, store_path) in &self.store_paths {
            let path_str = dirpath.to_string() + "/" + store_path;
            let path = Path::new(&path_str);

            if path.exists() {
                store_type_dirs.push(store_path.clone()); // returns only the name of the store type ("upload", "download", "remove" ...)
            }
        }
        if store_type_dirs.is_empty() {
            return None;
        }
        return Some(store_type_dirs);
    }

    /* todo get store type dir, location (hard link file) */
    fn get_store_type_filepath(&self, store: &StorageType, file_id: &str, store_type_dir: Option<&str>) -> Result<String> {
        let storage_dir: String = if let Some(dir) = store_type_dir {
            dir.to_string()
        } else if let Some(dir) = self.get_store_type_dir(&store) {
            dir
        } else {
            return Err(Box::new(MyError::new("Could not get location for storage type")));
        };

        return Ok(storage_dir + "/" + &file_id);
    }

}

impl filesystem::UserDiskFilesystem for CacheFS {
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: &[u8], storage_type: Option<StorageType>) -> Option<Box<dyn Error + Send>>{
        // todo encryption

        if !self.is_cur_dir_home_dir() { // todo useless if we check it in class instantiation (function `new`) ?
            return Some(Box::new(MyError::new("Current directory should be home directory of the filesystem")));
        }
        let dirpath = self.get_default_dirpath(file_id);
        let filepath = self.get_default_filepath(file_id);

        { // create and write file
            if let Some(err) = self.create_dir(&dirpath) {
                return Some(err);
            }
            match std::fs::File::create(&filepath) {
                Ok(mut file) => {
                    match self.set_file_content(&mut file, content, &file_id) {
                        Some(err) => {
                            return Some(err);
                        }
                        None => {}
                    }
                }
                Err(err) => {
                    return Some(Box::new(MyError::new(&(err.to_string()))));
                }
            }
        }
        if let Some(store_type) = storage_type {
            if let Some(store_type_dir) = self.get_store_type_dir(&store_type) {
                if let Some(err) = self.create_dir(&store_type_dir) {
                    return Some(err);
                }
                match self.get_store_type_filepath(&store_type, file_id, Some(&store_type_dir)) {
                    Ok(store_type_filepath) => {
                        if let Some(err) = self.create_hardlink( &filepath, &store_type_filepath) {
                            return Some(err);
                        }
                        /*
                        not great to create a symlink each time (especially when it's only to know the store type which can only be upload, download or remove) :
                        creates a new file inode
                        uses a few bytes for the filepath
                         */
                        self.create_symlink(&store_type_dir,
                            &(dirpath.to_string() + "/" + &store_type_dir),
                            Some(true));
                    }
                    Err(err) => {
                        return Some(err);
                    }
                }
            }
        }
        // create directory for user and disk (each id has to have a directory)
        if let Some(err) = self.create_dir(&(self.get_user_filepath(user_id, ""))) {
            return Some(err);
        }
        if let Some(err) = self.create_hardlink( &filepath, &(self.get_user_filepath(user_id, file_id))) {
            return Some(err);
        }
        if let Some(err) = self.create_symlink(&(self.get_user_filepath(user_id, "")), /* todo change with a function */&(dirpath.to_string() + "/user"), Some(true)) {
            return Some(err);
        }
        if let Some(err) = self.create_dir(&(self.get_disk_filepath(disk_id, ""))) {
            return Some(err);
        }
        if let Some(err) = self.create_hardlink( &filepath, &(self.get_disk_filepath(disk_id, file_id))) {
            return Some(err);
        }
        if let Some(err) = self.create_symlink(&(self.get_disk_filepath(disk_id, "")), /* todo change with a function */&(dirpath.to_string() + "/disk"), Some(true)) {
            return Some(err);
        }
        None
    }

    /* todo : don't require user_id and disk_id anymore */
    fn remove_file(&self, file_id: &str) -> Option<Box<dyn Error + Send>>{

        // todo useless get disk path instead
        // same fo user
        let disk_id = self.get_file_disk(file_id);
        if let Err(err) = disk_id {
            return Some(err)
        }
        let user_id = self.get_file_user(file_id);
        if let Err(err) = user_id {
            return Some(err)
        }
        /*
         *  remove all storage_type locations
         *  has to be done first because get_store_type_dirs_from_file retrieves info from file directory, which is also removed in the current function, later
         */
        if let Some(store_type_dirs) = self.get_store_type_dirs_from_file(file_id) {
            for store_type_dir in store_type_dirs {
                match self.get_store_type_filepath(&StorageType::None/* use store_type_dir instead of specifying the type */, file_id, Some(&store_type_dir)) {
                    Ok(store_type_filepath) => {
                        if let Err(err) = std::fs::remove_file(&store_type_filepath) {
                            return Some(Box::new(MyError::new(&(err.to_string()))));
                        }
                    }
                    Err(err) => {
                        return Some(err);
                    }
                }
            }
        }
        if let Err(err) = std::fs::remove_dir_all(&self.get_default_dirpath(file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_disk_filepath(&disk_id.unwrap(), file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_user_filepath(&user_id.unwrap(), file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }

        None
    }

    // get

    // get_files_disks returns map with key: disk_id, value: map with key: file_id as string, value: content as vector of u8
    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>>{
        // todo replicate for vault fs
        let mut files_disks: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
        // iterate through dirs in 'disks' dir, then iterate through  to fill up the map

        if let Ok(entries) = std::fs::read_dir(self.get_disk_filepath("", "")) {
            for entry in entries {
                if let Ok(disk_entry) = entry {
                    if let Some(diskname) = disk_entry.path().file_name() {
                        if let Some(diskname) = diskname.to_str() {
                            match self.get_disk_files(diskname) {
                                Ok(disk_files) => {
                                    files_disks.insert(String::from(diskname), disk_files);
                                }
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        }
                    }
                }
            }
            // todo test
            return Ok(files_disks);
        }
        /* todo format macro like for my_eprintln ? */
        return Err(Box::new(MyError::new(format!("Line {}, {}: Could not read dir : '{}'", line!(), file!(), self.get_disk_filepath("", "")).as_str())));
    }

    /// Returns map with key: file_id as string, value: content as vector of u8
    fn get_user_files(&self, user_id: &str) -> Result<HashMap<String, Vec<u8>>>{
        // todo replicate for vault fs
        let mut files: HashMap<String, Vec<u8>> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.get_user_filepath(user_id, "")) {
            for entry in entries {
                if let Ok(file_entry) = entry {
                    if let Some(filepath) = file_entry.path().to_str() {

                        // println!("file path : {}", basename(filepath));
                        match self.get_fileid_from_path(filepath) {
                            Ok(fileid) => {

                                match self.get_file_content_from_filepath(filepath, &fileid) {
                                    Ok(content) => {
                                        files.insert(fileid /* filename of link */, content);
                                    }
                                    Err(err) => {
                                        return Err(err);
                                    }
                                }
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    }
                }
            }
        } else {
            return Err(Box::new(MyError::new(/* todo format macro like for my_eprintln ? */&format!("Line {}, {}: Could not get user files, user '{}' may not exist yet", line!(), file!(), user_id))));
        }
        return Ok(files);
    }

    fn get_all_files_store_types(&self) -> Result<HashMap<String, Vec<maestro_vault::StorageType>>> {
        let mut store_types: HashMap<String, Vec<maestro_vault::StorageType>> = HashMap::new();
/* todo refactor, with adding files with several store types  */

        // check all dirs in list of store_types
        // in a loop, all the same
        for (store_type, store_path) in &self.store_paths {
            if let Ok(entries) = std::fs::read_dir(store_path) {
                for entry in entries {
                    if let Ok(file_entry) = entry {
                        if let Some(filepath) = file_entry.path().to_str() {
                            match self.get_fileid_from_path(filepath) {
                                Ok(file_id) => {
                                    // check if file is already in map
                                    match store_types.entry(file_id) {
                                        // if yes, simply add store type
                                        std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                                            let entry_store_types = occupied_entry.get_mut();
                                            /* todo find a better way to do that */
                                            entry_store_types.push(store_type.clone());
                                            *occupied_entry.get_mut() = entry_store_types.clone();
                                        }
                                        // if no, create map key value with value as vector
                                        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                                            vacant_entry.insert(vec!(store_type.clone()));
                                        }
                                    }
                                }
                                Err(err) => {
                                    my_eprintln!("Could not retrieve file id from filepath '{}' : {}", filepath, err.to_string());
                                }
                            }
                        }
                    } else if let Err(err) = entry {
                        eprintln!("Line {} in {} : {}", line!(), file!(), err.to_string());
                    }
                }
            }
        }
        if store_types.is_empty() {
            return Err(Box::new(MyError::new("Could not get all file store types")));
        }
        return Ok(store_types);
    }

    fn get_file_store_types(&self, file_id: &str) -> Result<Vec<maestro_vault::StorageType> /* todo could be several ones ? */> {
        /* todo redo */
        let mut store_types: Vec<maestro_vault::StorageType> = vec![];

        for store_type in self.store_paths.keys() {
            match self.get_store_type_filepath(store_type, file_id, None) {
                Ok(store_filepath) => {
                    /* if file exists */
                    if Path::new(&store_filepath).exists() {
                        store_types.push(store_type.clone());
                        continue;
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(store_types);
        // return Err(Box::new(MyError::new("Could not get file store type")));
    }

    fn get_files_store_types(&self, file_ids: Vec<&str>) -> Result<Vec<Vec<maestro_vault::StorageType>>> {
        let mut store_types: Vec<Vec<maestro_vault::StorageType>> = vec![];
        let empty_store_type_vec: Vec<maestro_vault::StorageType> = vec![maestro_vault::StorageType::None];

        for file_id in file_ids {
            match self.get_file_store_types(file_id) {
                Ok(file_store_types) => {
                    store_types.push(file_store_types);
                }
                Err(err) => {
                    eprintln!("Line {} in {} : {} : Couldn't get store type of file with id {}", line!(), file!(), err.to_string(), file_id);
                    store_types.push(empty_store_type_vec.clone());
                }
            }
        }
        /* if store_types.is_empty() {
            return Err(Box::new(MyError::new("Could not get file store type")));
        } */
        return Ok(store_types);
    }

    fn get_store_type_files(&self, store_type: StorageType) -> Result<Vec<String>> {
        let mut store_type_files: Vec<String> = vec![];

        /* todo test */
        if let Some(store_type_dir) = self.get_store_type_dir(&store_type) {
            /* get all files from dir */
            if let Ok(entries) = std::fs::read_dir(store_type_dir) {
                for entry in entries {
                    if let Ok(file_entry) = entry {
                        if let Some(filename) = file_entry.path().file_name() {
                            if let Some(filename) = filename.to_str() {
                                store_type_files.push(filename.to_string());
                            }
                        }
                    }
                }
            }
        }
        return Ok(store_type_files);
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