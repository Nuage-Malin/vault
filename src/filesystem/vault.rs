use crate::filesystem;

use std::collections::HashMap;
use std::error::Error;
use std::any::Any;
use std::io::Write;
// use std::path::Path;

use super::UserDiskFilesystem;
use super::MyError;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct VaultFS{}

impl filesystem::UserDiskFilesystem for VaultFS {
    // todo create and move to directory 'vault'
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: Vec<u8>, _: Option<i32>) -> Option<Box<dyn Error + Send>> {
        if !self.is_cur_dir_home_dir() {
            return Some(Box::new(MyError::new("Current directory should be home directory of the filesystem")));
        }
        let filepath = self.get_default_filepath(&file_id);

        match std::fs::File::create(&filepath) {
            Ok(mut file) => {
                match file.write_all(&content) {
                    Ok(_) => {
                    }
                    Err(err) => {
                        return Some(Box::new(MyError::new(&(err.to_string()))));
                    }
                }
            }
            Err(err) => {
                return Some(Box::new(MyError::new(&(err.to_string()))));
            }
        }
        if let Some(err) = self.create_dir(&(self.get_user_filepath(user_id, ""))) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_hardlink(&filepath, &self.get_user_filepath(&user_id, &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_dir(&(self.get_disk_filepath(disk_id, ""))) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Some(err) = self.create_hardlink(&filepath, &self.get_disk_filepath(&disk_id, &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        None
    }

    fn remove_file(&self, file_id: &str, user_id: &str, disk_id: &str) -> Option<Box<dyn Error + Send>> {
        if let Err(err) = std::fs::remove_file(self.get_default_filepath(&file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_disk_filepath(&disk_id, &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_user_filepath(&user_id, &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        return None
    }

    fn set_file_content(&self, file_id: &str, content: Vec<u8>) -> Option<Box<dyn Error>> {
        let filepath = self.get_default_filepath(file_id);
        let res = std::fs::write(&filepath, &content);

        match res {
            Ok(_) => {None}
            Err(err) => {
                return Some(Box::new(MyError::new(&(err.to_string()))));

            }
        }
    }

    fn get_file_content(&self, file_id: &str) -> Result<Vec<u8>> {
        let res = std::fs::read(self.get_default_filepath(file_id));

        match res {
            Ok(content) => {
                Ok(content)
            }
            Err(err) => {
                Err(Box::new(err))
            }
        }
    }

    fn get_disk_files(&self, disk_id: &str) -> Result<HashMap<String, Vec<u8>>> {
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

    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>> {
        let files_disks: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();

        return Ok(files_disks);

    }

    fn get_user_files(&self, _user_id: &str) -> Result<HashMap<String, Vec<u8>>> {
        let files: HashMap<String, Vec<u8>> = HashMap::new();

        return Ok(files);
    }


    fn get_home_dir(&self) -> String {
        String::from("vault_fs")
    }

///
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl VaultFS {
    pub fn new() -> Result<Self> {
        let vault_fs = VaultFS{};

        vault_fs.cd_home_dir();
        if let Some(err) = vault_fs.create_dir(&vault_fs.get_default_filepath("")) {
            return Err(err);
        }
        if let Some(err) = vault_fs.create_dir(&vault_fs.get_user_filepath("", "")) {
            return Err(err);
        }
        if let Some(err) = vault_fs.create_dir(&vault_fs.get_disk_filepath("", "")) {
            return Err(err);
        }
        return Ok(vault_fs);
    }

    // fn get_user_symlink_base_path(&self) {
    // }
}

impl Drop for VaultFS {
    fn drop(&mut self) {
        self.cd_home_dir_parent();
    }
}