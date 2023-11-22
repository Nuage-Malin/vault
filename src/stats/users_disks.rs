
use crate::models::users_disks::{UserDiskInfo, ApproxUserDiskInfo, UserDiskUpdate, ApproxUserDiskUpdate, DiskAction, DiskWakeup};
use bson::doc;
use mongodb::{Client, Collection, results::InsertOneResult, options::FindOneOptions};
use bson::oid::ObjectId;

extern crate sysinfo;
use sysinfo::SystemExt;

use std::str::FromStr;
use std::error::Error;
use std::env;
use crate::filesystem::error::MyError;
use crate::my_eprintln;

type Result<T> = std::result::Result<T, Box<dyn Error + Send>>;

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
        match self.user_disk_update.insert_one(UserDiskUpdate {
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
        }, None).await {
            Ok(res) => {
                return Ok(res)
            }
            Err(r) => {
                return Err(Box::new(MyError::new(&r.to_string())));
            }
        }
    }

    pub async fn disk_used_memory_update(&self, disk_update: ApproxUserDiskInfo) -> Result<InsertOneResult>
    {
        let options = FindOneOptions::builder()
            .sort(doc! { "startup.date": -1 })
            .build();
        let disk_wakeup = self.disk_wakeup.find_one(
            doc!{"diskId": disk_update.disk_id /* "shutdown": None */}, options
            // error here : disk_update.disk_id is object id but in db is disk id
        ).await;
        match disk_wakeup {
            Ok(res) => {
                match res {
                    Some(disk_wakeup) => {
                        let mut system = sysinfo::System::new_all();

                        system.refresh_all();
                        let my_user_disk_info = UserDiskInfo{
                            _id: ObjectId::new(),
                            disk_id: Some(disk_update.disk_id),
                            user_id: Some(disk_update.user_id),
                            disk_wakeup: Some(disk_wakeup._id),
                            used_memory: system.used_memory(),
                            created_at: bson::DateTime::now()
                        };
                        match self.user_disk_info.insert_one(my_user_disk_info, None).await {
                            Ok(res) => {
                                return Ok(res);
                            }
                            Err(r) => {
                                return Err(Box::new(MyError::new(&r.to_string())));
                            }
                        }
                    }
                    None => {
                        return Err(Box::new(MyError::new("Error TODO"))); // todo log error
                    }
                }
            }
            Err(r) => {
                my_eprintln!("{}", r); // todo print this error only once (choose between this call and the one later)
                return Err(Box::new(MyError::new(&r.to_string())));
            }
        }
        // &format!("Line {} in {} : Could not get file store type", line!(), file!())
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
                my_eprintln!("Could not insert disk update log");
                my_eprintln!("{}", err);
            }
        };
        let disk_used_memory = self.disk_used_memory_update(disk_info).await;

        match disk_used_memory {
            Ok(_) => {}
            Err(err) => {
                my_eprintln!("Could not insert updated used disk memory :");
                my_eprintln!("{}", err);
            }
        };
    }
}