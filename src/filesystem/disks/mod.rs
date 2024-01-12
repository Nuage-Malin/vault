mod tests;

extern crate sysinfo;

use sysinfo::SystemExt;
use sysinfo::DiskExt;
use std::error::Error;
use super::MyError;

use crate::models::users_disks::AvailableDisk;
use crate::my_eprintln;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

const DISK_IDS_DIR: &str = "/dev/disk/by-uuid/";

#[derive(Debug, Default)]
pub struct CurrentDisks{
    sys: sysinfo::System
}

impl CurrentDisks {
    pub fn new() -> Self {
        let mut my_current_disks = CurrentDisks{sys: sysinfo::System::new_all()};
        my_current_disks.sys.refresh_all();

        return my_current_disks;
    }

/*     fn disks() {}

    fn disk_paths() -> HashMap<disk, String> {}

    fn disk_ids() -> HashMap<disk, String> {}

    fn disk_available_sizes() -> HashMap<disk, Int> {}
 */

    fn get_compatible_disks(&self) -> Vec<AvailableDisk> {
        let mut compatible_disks: Vec<AvailableDisk> = Vec::new();

        // TODO ? partitions instead of disks ?
        // self.sys.refresh_partitions();
        // let partitions = self.sys.get_partitions();
        // for partitions in partitions.values() {
        // }
        for disk in self.sys.disks() {
            if disk.file_system() != [b'e', b'x', b't', b'4'] {continue;}
            if disk.total_space() <= 0 {continue;}
            if disk.available_space() <= 0 {continue;}

            let device_name = if let Some(string) = disk.name().to_str() {String::from(string)} else {continue;};
            let mount_point = if let Some(string) = disk.mount_point().to_str() {String::from(string)} else {continue;};
            let mut index_to_replace: Option<usize> = None;

            for (counter, compatible_disk) in compatible_disks.iter().enumerate() {
                if compatible_disk.device_name == device_name { // keep only the one that is '/'
                    if compatible_disk.mount_point == "/" {
                        continue;
                    } else if mount_point == "/" {
                        index_to_replace = Some(counter);
                    } else {
                        continue;
                    }
                }
            }
            if let Some(index) = index_to_replace {
                compatible_disks.remove(index);
            }
            compatible_disks.push(AvailableDisk{
                uid: String::new(),
                type_: disk.kind(),
                device_name: device_name,
                mount_point: mount_point,
                total_space: disk.total_space() as usize,
                available_space: disk.available_space() as usize
            });
        }
        return compatible_disks
    }


    fn assign_uuid_to_corresponding_disk(&self, disks: &mut Vec<AvailableDisk>, device_name: &str,  uuid: &str) {
        for disk in disks {
            if device_name == disk.device_name {
                disk.uid = String::from(uuid);
            }
        }
    }

    /// if file is symlink :
    ///      check to what it points to,
    ///      if it corresponds to device_name of an existing disk, set its uid
    fn get_disk_id(&self, disks: &mut Vec<AvailableDisk>, disk_entry: std::fs::DirEntry) -> Option<Box<dyn Error + Send>> {

        match disk_entry.file_type() {
            Ok(file_type) => {
                if !file_type.is_symlink() {
                    return Some(Box::new(MyError::new("Could not retrieve disk id for entry")));
                }
            }
            Err(r) => {
                return Some(Box::new(MyError::new(&r.to_string())));
            }
        }

        let current_device_path: String;

        match std::fs::read_link(disk_entry.path().as_path()) {
            Ok(path) => {
                if !path.is_absolute() {
                    if let Some(path) = path.to_str() {
                        current_device_path = String::from(DISK_IDS_DIR) + path;
                    } else {
                        return Some(Box::new(MyError::new("Could not get path from disk device link")));
                    }
                } else {
                    unimplemented!("Device is not a symbolic link ?")
                }
            }
            Err(r) => {
                return Some(Box::new(MyError::new(&r.to_string())));
            }
        }

        let current_device_name: String;

        match std::fs::canonicalize(&current_device_path) {
            Ok(path) => {
                if let Some(device_name) = path.to_str() {
                    current_device_name = String::from(device_name);
                } else {
                    return Some(Box::new(MyError::new("Could not get path from absolute device filepath")));
                }
            }
            Err(r) => {
                return Some(Box::new(MyError::new(&r.to_string())));
            }
        }

        if let Some(uuid) = disk_entry.path().file_name() {
            if let Some (uuid) = uuid.to_str() {
                self.assign_uuid_to_corresponding_disk(disks, &current_device_name, uuid);
                return None;
            }
        }
        return Some(Box::new(MyError::new("Could not get uuid from device disk path")));
    }

    fn get_disk_ids(&self, disks: &mut Vec<AvailableDisk>) {

        if let Ok(entries) = std::fs::read_dir(DISK_IDS_DIR) {
            for entry in entries {
                if let Ok(disk_entry) = entry {
                    if let Some(r) = self.get_disk_id(disks, disk_entry) {
                        my_eprintln!("Error ! {}", r);
                    }
                }
            }
        }
    }

    /// Get all disks and their corresponding device (with partition number: e.g /dev/sda7),
    /// parse /dev/disk/by-uuid to obtain a unique identifier (and its associated device name),
    /// link uid with device name and store it in vec of Disks
    pub fn get_disks(&self) -> Vec<AvailableDisk>{
        let mut compatible_disks = self.get_compatible_disks();

        self.get_disk_ids(&mut compatible_disks);

        return compatible_disks;
    }

    pub fn select_disk_for_file(&self, file_size: usize) -> Result<AvailableDisk> {
        let mut disk_with_biggest_available_space = AvailableDisk{
            uid: String::new(),
            type_: sysinfo::DiskKind::Unknown(0),
            device_name: String::from(""),
            mount_point: String::from(""),
            total_space: 0,
            available_space: 0};

        for disk in &self.get_disks() {
            if file_size < disk.available_space {
                if disk.available_space > disk_with_biggest_available_space.available_space {
                    disk_with_biggest_available_space = disk.clone();
                }
            }
            println!("\n\n");
        }
        if disk_with_biggest_available_space.available_space > 0 {
            return Ok(disk_with_biggest_available_space);
        }
        return Err(Box::new(MyError::new("Could not find disk with sufficient size")));
    }

    // todo get ID from /dev/disk
    // https://www.baeldung.com/linux/dev-directory
    // https://superuser.com/questions/558156/what-does-dev-sda-in-linux-mean
    // https://unix.stackexchange.com/questions/86764/understanding-dev-disk-by-folders


}