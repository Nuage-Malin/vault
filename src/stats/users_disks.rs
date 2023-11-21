
use crate::models::users_disks::{UserDiskInfo, ApproxUserDiskInfo, UserDiskUpdate, ApproxUserDiskUpdate, DiskAction, DiskWakeup};
use bson::{doc};
use mongodb::{Client, Collection, results::InsertOneResult, error::Result, options::FindOneOptions};
use bson::oid::ObjectId;

extern crate sysinfo;
use sysinfo::{SystemExt, Disk};

use std::str::FromStr;
use std::env;
// use std::error::Error;

pub struct MongoRepo {
    user_disk_info: Collection<UserDiskInfo>,
    user_disk_update: Collection<UserDiskUpdate>,
    disk_wakeup: Collection<DiskWakeup>,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let client_uri = env::var("MONGODB_URI").expect("MONGODB_URI not set.");
        // println!("{}", client_uri);

        let client = Client::with_uri_str(client_uri).await.unwrap();
        let db = client.database("logs");
        let user_disk_info: Collection<UserDiskInfo> = db.collection("userDiskInfo");
        let user_disk_update: Collection<UserDiskUpdate> = db.collection("userDiskUpdate");
        let disk_wakeup: Collection<DiskWakeup> = db.collection("diskWakeup");

        // println!("fetching databases...");
        // for val in client.list_databases(None, None).await.unwrap() {
            // println!("{:?}", val);
        // }

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
                DiskAction::WRITE => "w".to_string(),
                DiskAction::CREATE => "c".to_string(),
                DiskAction::DELETE => "d".to_string()
            },
            created_at: bson::DateTime::now()
        }, None).await
    }

    pub async fn disk_used_memory_update(&self, disk_update: ApproxUserDiskInfo) -> Result<InsertOneResult>
    {
        let options = FindOneOptions::builder()
            .sort(doc! { "startup.date": -1 })
            .build();
        let disk_wakeup = self.disk_wakeup.find_one(
            doc!{"diskId": disk_update.disk_id, /* "shutdown": None */}, options
            // error here : disk_update.disk_id is object id but in db is disk id
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

    pub async fn update_disk_logs(&self, disk_id: &str, user_id: &str, file_id: &str, action: DiskAction)
    {
        let disk_update = ApproxUserDiskUpdate{
            disk_id: ObjectId::from_str(disk_id).unwrap(),
            user_id: ObjectId::from_str(user_id).unwrap(),
            file_id: ObjectId::from_str(file_id).unwrap(),
            action: action
        };
        let disk_info = ApproxUserDiskInfo{
            disk_id: ObjectId::from_str(disk_id).unwrap(),
            user_id: ObjectId::from_str(user_id).unwrap()
        };
        let disk_update = self.disk_update_insert(disk_update).await;

        match disk_update {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Could not insert disk update log");
                eprintln!("{}", err);
            }
        };
        let disk_used_memory = self.disk_used_memory_update(disk_info).await;

        match disk_used_memory {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Could not insert updated used disk memory :");
                eprintln!("{}", err);
            }
        };
    }
}