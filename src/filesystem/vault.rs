
use std::collections::HashMap;
use std::error::Error;
use std::any::Any;

use crate::filesystem;
use crate::models::grpc::maestro_vault::{self, StorageType};

use crate::models::users_disks::AvailableDisk;
use crate::filesystem::disks::CurrentDisks;

use super::UserDiskFilesystem;
use super::MyError;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

#[derive(Debug)]
pub struct VaultFS{
    disks: CurrentDisks,
}

impl filesystem::UserDiskFilesystem for VaultFS {
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: &[u8], _: Option<StorageType>) -> Option<Box<dyn Error + Send>> {
        // AvailableDisk{uid: String::new(), type_: sysinfo::DiskKind::Unknown(0)};
        let mut disk: AvailableDisk = AvailableDisk{uid: String::new(),
                                                type_: sysinfo::DiskKind::Unknown(0),
                                                device_name: String::new(),
                                                mount_point: String::from("/"),
                                                total_space: 0,
                                                available_space: 0};

        match self.disks.select_disk_for_file(content.len()) {
            Ok(available_disk) => {
                disk = available_disk;
            }
            Err(_r) => {
            }
        }

        if !self.is_cur_dir_home_dir() { // todo useless if we check it in class instantiation (function `new`) ?
            return Some(Box::new(MyError::new("Current directory should be home directory of the filesystem")));
        }
        let dirpath = self.get_default_dirpath(file_id);
        let filepath = self.get_default_filepath(file_id);
        let mut actual_filepath: String = filepath.clone();

        if disk.mount_point != "/" { // put actual file in specific disk if non-default disk
            actual_filepath = String::from(disk.mount_point) + &filepath;
            match std::fs::File::create(&actual_filepath) {
                Ok(mut file) => {
                    match self.set_file_content(&mut file, &content, &file_id) {
                        Some(err) => {
                            return Some(err);
                        }
                        None => {}
                    }
                }
                Err(_err) => {
                    // if err.to_string() == "Permission denied (os error 13)" {
                    actual_filepath = filepath.clone();
                    // } else {
                        // return Some(Box::new(MyError::new(&(err.to_string()))));
                    // }
                }
            }
        }

        { // create file directory
            if let Some(err) = self.create_dir(&dirpath) {
                return Some(err);
            }
        }

        { // create and write file
            if actual_filepath != filepath { // file has been written on specific disk
                if let Some(err) = self.create_hardlink( &actual_filepath, &filepath) {
                    return Some(err);
                }
            } else { // file is stored on the default disk
                match std::fs::File::create(&filepath) {
                    Ok(mut file) => {
                        match self.set_file_content(&mut file, &content, &file_id) {
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

    fn remove_file(&self, file_id: &str) -> Option<Box<dyn Error + Send>> {
        let disk_id = self.get_file_disk(&file_id);
        if let Err(err) = disk_id {
            return Some(err)
        }
        let user_id = self.get_file_user(&file_id);
        if let Err(err) = user_id {
            return Some(err)
        }

        if let Err(err) = std::fs::remove_dir_all(self.get_default_dirpath(&file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_disk_filepath(&disk_id.unwrap(), &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        if let Err(err) = std::fs::remove_file(&self.get_user_filepath(&user_id.unwrap(), &file_id)) {
            return Some(Box::new(MyError::new(&(err.to_string()))));
        }
        return None
    }

    fn get_files_disks(&self) -> Result<HashMap<String, HashMap<String, Vec<u8>>>> {
        let files_disks: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();

        return Ok(files_disks);

    }

    fn get_user_files(&self, user_id: &str) -> Result<HashMap<String, Vec<u8>>> {
        let mut files: HashMap<String, Vec<u8>> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(self.get_user_filepath(user_id, "")) {
            for entry in entries {
                if let Ok(file_entry) = entry {
                    if let Some(filepath) = file_entry.path().to_str() {

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
        } else {
            return Err(Box::new(MyError::new(/* todo format macro like for my_eprintln ? */&format!("Line {}, {}: Could not get user files, user '{}' may not exist yet", line!(), file!(), user_id))));
        }
        return Ok(files);
    }

    fn get_all_files_store_types(&self) -> Result<HashMap<String, Vec<maestro_vault::StorageType>>> {
        return Err(Box::new(MyError::new("Could not get file store types : method not implemented for vault")));
    }

    fn get_file_store_types(&self, _file_id: &str) -> Result<Vec<maestro_vault::StorageType>> {
        return Err(Box::new(MyError::new("Could not get file store type : method not implemented for vault")));
    }

    fn get_files_store_types(&self, _file_id: Vec<&str>) -> Result<Vec<Vec<maestro_vault::StorageType>>> {
        return Err(Box::new(MyError::new("Could not get file store types : method not implemented for vault")));
    }

    fn get_store_type_files(&self, _store_type: StorageType) -> Result<Vec<String>> {
        return Err(Box::new(MyError::new("Could not get file store types : method not implemented for vault")));
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
        let vault_fs = VaultFS{disks: CurrentDisks::new()};

        vault_fs.cd_home_dir();
        if let Some(err) = vault_fs.create_dir(&vault_fs.get_default_dirpath("")) {
            return Err(err);
        }
        if let Some(err) = vault_fs.create_dir(&vault_fs.get_user_filepath("", "")) {
            return Err(err);
        }
        if let Some(err) = vault_fs.create_dir(&vault_fs.get_disk_filepath("", "")) {
            return Err(err);
        }

        let disks = vault_fs.disks.get_disks();

        for disk in disks {
            if disk.mount_point != "/" {
                if let Some(_err) = vault_fs.create_dir(&(String::from(&disk.mount_point) + &vault_fs.get_default_dirpath(""))) {
                    eprintln!("Could not write to disk '{}' mounted at '{}'", &disk.device_name, &disk.mount_point);
                    // return Err(err);
                }
            }
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