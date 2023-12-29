
#[cfg(test)]
mod tests {

use crate::filesystem::disks::CurrentDisks;
use crate::models::users_disks::AvailableDisk;
use crate::my_eprintln; /* todo ask nell why this is directly crate ? */

use lazy_static::lazy_static;

lazy_static! {
    static ref DISKS: CurrentDisks = CurrentDisks::new();
}

#[test]
fn get_disks_test() {
    let disks: Vec<AvailableDisk> = DISKS.get_disks();

    assert_ne!(disks.len(), 0);
    for disk in disks {
        assert_ne!(disk.available_space, 0);
        assert_ne!(disk.total_space, 0);
        assert_ne!(disk.uid, String::new());
        assert_ne!(disk.device_name, String::new());
    }
}

#[test]
fn get_compatible_disks_test() {
    let disks: Vec<AvailableDisk> = DISKS.get_compatible_disks();

    assert_ne!(disks.len(), 0);
    for disk in disks {
        assert_ne!(disk.available_space, 0);
        assert_ne!(disk.total_space, 0);
        assert_eq!(disk.uid, String::new());
        assert_ne!(disk.device_name, String::new());
    }
}

#[test]
fn select_disk_for_file_test() {
    let file_sizes: [u64; 7] = [2, 10, 100, 1000, 10000, 100000, 1000000];

    for file_size in file_sizes {
        match DISKS.select_disk_for_file(file_size) {
            Ok(disk) => {
                assert_ne!(disk.available_space, 0);
                assert_ne!(disk.total_space, 0);
                assert_ne!(disk.uid, String::new());
                assert_ne!(disk.device_name, String::new());
            }
            Err(r) => {
                my_eprintln!("`select_disk_for_file` failed while the file size was {} bytes, error message is:", file_size);
                my_eprintln!("Error: {}", r);
                assert!(false);
            }
        }
    }
}

#[test]
fn select_disk_for_file_too_big_returns_error_test() {
    let file_size: u64 = u64::MAX;

    match DISKS.select_disk_for_file(file_size) {
        Ok(_) => {
            my_eprintln!("`select_disk_for_file` succeeded while the file size was {} bytes, it should be bigger than the available disk size ?", file_size);
            assert!(false);
        }
        Err(r) => {
            assert_eq!(r.to_string(), "Could not find disk with sufficient size");
        }
    }
}
}