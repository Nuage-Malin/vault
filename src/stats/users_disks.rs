
use crate::models::users_disks::{UserDiskInfo, ApproxUserDiskInfo, UserDiskUpdate, ApproxUserDiskUpdate, DiskAction, DiskWakeup};
use bson::{doc};
use mongodb::{Client, Collection, results::InsertOneResult, error::Result, options::FindOneOptions};

extern crate sysinfo;
use sysinfo::SystemExt;

use std::env;

pub struct MongoRepo {
    user_disk_info: Collection<UserDiskInfo>,
    user_disk_update: Collection<UserDiskUpdate>,
    disk_wakeup: Collection<DiskWakeup>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let client_uri = env::var("MONGODB_URI").expect("MONGODB_URI not set.");
        println!("{}", client_uri);

        let client = Client::with_uri_str(client_uri).await.unwrap();
        let db = client.database("logs");
        let user_disk_info: Collection<UserDiskInfo> = db.collection("userDiskInfo");
        let user_disk_update: Collection<UserDiskUpdate> = db.collection("userDiskUpdate");
        let disk_wakeup: Collection<DiskWakeup> = db.collection("diskWakeup");

        println!("fetching databases...");
        for val in client.list_databases(None, None).await.unwrap() {
            println!("{:?}", val);
        }

        MongoRepo { user_disk_info, user_disk_update, disk_wakeup}
    }

    pub async fn disk_update_insert(&self, disk_update: ApproxUserDiskUpdate) -> Result<InsertOneResult>
    {
        self.user_disk_update.insert_one(UserDiskUpdate {
            disk_id: Some(disk_update.disk_id),
            user_id: Some(disk_update.user_id),
            file_id: Some(disk_update.file_id),
            action: match disk_update.action {
                DiskAction::READ => "r".to_string(),
                DiskAction::CREATE => "c".to_string(),
                DiskAction::DELETE => "d".to_string()
            },
            created_at: bson::DateTime::now()
        }, None).await
    }

    pub async fn disk_update_used_memory(&self, disk_update: ApproxUserDiskInfo) -> Result<InsertOneResult>
    {

        println!("disk_id : {}", disk_update.disk_id);
        let options = FindOneOptions::builder()
            .sort(doc! { "startup.date": -1 })
            .build();
        let disk_wakeup = self.disk_wakeup.find_one(
            doc!{"diskId": disk_update.disk_id, /* "shutdown": None */}, options
        ).await?
        .expect("No previous disk wake up found");
        let mut system = sysinfo::System::new_all();

        system.refresh_all();
        // println!("Disk: {}", system.used_memory());
        self.user_disk_info.insert_one(UserDiskInfo{
            disk_id: Some(disk_update.disk_id),
            user_id: Some(disk_update.user_id),
            disk_wakeup: Some(disk_wakeup._id),
            used_memory: system.used_memory(),
            created_at: bson::DateTime::now()
        }, None).await
    }
}