use crate::filesystem;

use std::collections::HashMap;
use std::error::Error;
use std::any::Any;
use std::io::Write;

use super::UserDiskFilesystem;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct VaultFS{}


impl filesystem::UserDiskFilesystem for VaultFS {
    // todo create and move to directory 'vault'
    fn create_file(&self, file_id: &str, user_id: &str, disk_id: &str, content: Vec<u8>, _: Option<i32>) -> Option<Box<dyn Error>> {
        let filepath = self.get_default_filepath(&file_id);
        // todo create directory
        println!("filepath : {}", filepath);
        match std::env::current_dir() {
            Ok(cur) => {
                if let Some(act_dir) = cur.to_str() {
                    println!("pwd : {}", act_dir);
                }
            }
            Err(err) => {
                eprintln!("error from current dir : {}", err);
            }
        }
        let create_res = std::fs::File::create(&filepath);
        let res: std::io::Result<()>;

        match create_res {
            Ok(mut file) => {
                res = file.write_all(&content);
                match res {
                    Ok(_) => {}
                    Err(err) => {
                        return Some(Box::new(err));
                    }
                }
            }
            Err(err) => {
                return Some(Box::new(err));
            }
        }
        if let Some(err) = self.create_dir(&self.get_user_filepath(&user_id, "")) {
            return Some(err);
        }
        if let Some(err) = self.create_symlink( &(String::from("../../") + &filepath), &self.get_user_filepath(&user_id, &file_id)) {
            println!("hello error 3");
            println!("{}, {}", &filepath, self.get_user_filepath(&user_id, &file_id));

            return Some(err);
        }
        if let Some(err) = self.create_dir(&self.get_disk_filepath(&disk_id, "")) {
            return Some(err);
        }
        if let Some(err) = self.create_symlink( &(String::from("../../") + &filepath), &self.get_disk_filepath(&disk_id, &file_id)) {
            println!("hello error 4");
            println!("{}, {}", &filepath, self.get_disk_filepath(&user_id, &file_id));

            return Some(err);
        }
        None
    }

    fn remove_file(&self, file_id: &str) -> Option<Box<dyn Error>> {
        let res = std::fs::remove_file(self.get_default_filepath(file_id));
        // todo remove symlink(s)

        match res {
            Ok(_) => {None}
            Err(err) => {
                return Some(Box::new(err));
            }
        }
    }

    fn set_file_content(&self, file_id: &str, content: Vec<u8>) -> Option<Box<dyn Error>> {
        let filepath = self.get_default_filepath(file_id);
        let res = std::fs::write(&filepath, &content);

        match res {
            Ok(_) => {None}
            Err(err) => {
                return Some(Box::new(err));
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
    pub fn new() -> Self {
        // todo create dir if does not exist
        let vault_fs = VaultFS{};

        vault_fs.cd_home_dir();
        std::fs::create_dir(vault_fs.get_default_filepath(""));
        std::fs::create_dir(vault_fs.get_user_filepath("", "")); // todo handle error ?
        std::fs::create_dir(vault_fs.get_disk_filepath("", "")); // todo if error return None ?

        vault_fs
    }
}

impl Drop for VaultFS {
    fn drop(&mut self) {
        self.cd_home_dir_parent();
    }
}