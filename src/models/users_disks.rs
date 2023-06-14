
use bson::oid::ObjectId;
// use bson::oid::primitive;
use bson::{DateTime};
// use chrono::DateTime;
use serde::{Deserialize, Serialize};

/**
 * Add: On disk shutdown
 */
// TODO create a disk DB fed and checked by santaclaus to store disks to compare that
//  know free space of disk ...

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDiskInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
	pub disk_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
	pub user_id: Option<ObjectId>,
    // #[serde(skip_serializing_if = "Option::is_none")]
	pub disk_wakeup: Option<ObjectId>, // Ref to diskWakeup
	pub used_memory: u64, // Used memory by the user
	pub created_at: DateTime
}

pub struct ApproxUserDiskInfo {
	pub disk_id: ObjectId,
	pub user_id: ObjectId
}

#[derive(Debug)]
pub enum DiskAction { // todo protobuf with enum so that it is compatible with any other service
    READ,
    CREATE,
    DELETE
}

/**
 *  Add: On user read/write on a disk
*/
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDiskUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
	pub disk_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
	pub user_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
	pub file_id: Option<ObjectId>, // Not sure to keep this data
	pub action: String, // Enum: read | write | delete
	pub created_at: DateTime
}

/**
 * UserDiskUpdate with only input information,
 * without time (which will be set at time of insert)
 */
pub struct ApproxUserDiskUpdate {
	pub disk_id: ObjectId,
	pub user_id: ObjectId,
	pub file_id: ObjectId, // Not sure to keep this data
	pub action: DiskAction, // Enum: read | write | delete
}