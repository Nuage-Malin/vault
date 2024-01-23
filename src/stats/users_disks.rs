
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
    pub async fn init() -> Self
    {
        let client_uri = env::var("MONGO_URI").expect("MONGO_URI not set.");
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
            disk_id: disk_update.disk_id,
            user_id: disk_update.user_id,
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
            doc!{"diskId": disk_update.disk_id.clone()}, options
        ).await;

        match disk_wakeup {
            Ok(res) => {
                match res {
                    Some(disk_wakeup) => {
                        let mut system = sysinfo::System::new_all();

                        system.refresh_all();
                        let my_user_disk_info = UserDiskInfo{
                            _id: ObjectId::new(),
                            disk_id: disk_update.disk_id.clone(),
                            user_id: disk_update.user_id,
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
                                // todo instead of returning error then, when printing, appending line number and file name :
                                // append line number and filename to error when error occurs,
                                // then return that and print it such as
                            }
                        }
                    }
                    None => {
                        eprint!("DiskId: {}", line!());
                        return Err(Box::new(MyError::new("Didn't find any previous DiskWakeup"))); // todo log error
                        // todo instead of returning error then, when printing, appending line number and file name :
                        // append line number and filename to error when error occurs,
                        // then return that and print it such as
                    }
                }
            }
            Err(r) => {
                return Err(Box::new(MyError::new(&r.to_string())));
                // todo instead of returning error then, when printing, appending line number and file name :
                // append line number and filename to error when error occurs,
                // then return that and print it such as
            }
        }
        // &format!("Line {} in {} : Could not get file store type", line!(), file!())
    }

    pub async fn update_disk_logs(&self, disk_id: Option<String>, user_id: Option<String>, file_id: &str, action: DiskAction)
    {
        let my_user_id: Option<ObjectId> = if user_id.is_some() { Some(ObjectId::from_str(&user_id.unwrap()).unwrap())} else {None};
        let my_file_id: ObjectId;

        match ObjectId::from_str(file_id) {
            Ok(id) => {
                my_file_id = id;
            }
            Err(_) => {
                my_eprintln!("Incorrect file id, aborting update_disk_logs");
                return;
            }
        };

        let disk_update = ApproxUserDiskUpdate{
            disk_id: disk_id.clone(),
            user_id: my_user_id,
            file_id: my_file_id,
            action: action
        };
        let disk_info = ApproxUserDiskInfo{
            disk_id: disk_id.clone(),
            user_id: my_user_id
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