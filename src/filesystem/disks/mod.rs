extern crate sysinfo;
use sysinfo::SystemExt;
use sysinfo::DiskExt;
use sysinfo::Disk;

#[derive(Debug)]
pub struct CurrentDisks{
    sys: sysinfo::System
}

impl CurrentDisks {
    pub fn new() -> Self {
        let mut my_current_disks = CurrentDisks{sys: sysinfo::System::new_all()};
        my_current_disks.sys.refresh_all();

        my_current_disks
    }

/*     fn disks() {}

    fn disk_paths() -> HashMap<disk, String> {}

    fn disk_ids() -> HashMap<disk, String> {}

    fn disk_available_sizes() -> HashMap<disk, Int> {}
 */
    fn get_compatible_disks(&self) -> Vec<Disk> {
        let mut compatible_disks: Vec<Disk> = Vec::new();

        for disk in self.sys.disks() {
            // println!("{:?}", disk);
            // dbg!(disk);
            if disk.file_system() != [b'e', b'x', b't', b'4'] {
                continue;
            }
            if disk.total_space() < 0 {
                continue;
            }
            if disk.available_space() < 0 {
                continue;
            }
            println!("\n\n");

            compatible_disks.push(disk.clone());
        }
        return compatible_disks
    }

    pub fn select_disk_for_file(&self, file_size: u64) {
        for disk in self.get_compatible_disks() {
            println!("{:?}", disk);
            dbg!(disk);
            if file_size < disk.available_space() {
                println!("Enough space on this disk !");
            }
            println!("\n\n");
        }
    }
}