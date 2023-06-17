
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
    #[serde(rename = "diskId", skip_serializing_if = "Option::is_none")]
	pub disk_id: Option<ObjectId>,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
	pub user_id: Option<ObjectId>,
    #[serde(rename = "fileId", skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
	date: DateTime,

    #[serde(rename = "isManual")]
	is_manual: bool // todo change name at serialisation, with original notion name
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeriodInfo {
	consumption: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskWakeup {
    // #[serde(skip_serializing_if = "Option::is_none")]
	pub _id: ObjectId,
    #[serde(rename = "diskId")]
	pub disk_id: ObjectId, // Disk serial number
	pub startup: Event,
    // #[serde(rename = "periodInfo")]
	// period_info: PeriodInfo,
	pub shutdown: Event, // null until disk shutdown
    #[serde(rename = "periodInfo")]
	pub period_info: PeriodInfo // Disk consumption since disk startup, null until disk shutdown
}