
use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};
use sysinfo::DiskKind;

/**
 * Add: On disk shutdown
 */
// TODO create a disk DB fed and checked by santaclaus to store disks to compare that
//  know free space of disk ...

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDiskInfo {
	pub _id: ObjectId,
    #[serde(rename="diskId", skip_serializing_if = "Option::is_none")]
	pub disk_id: Option<ObjectId>,
    #[serde(rename="userId", skip_serializing_if = "Option::is_none")]
	pub user_id: Option<ObjectId>,
    #[serde(rename="diskWakeup")]
	pub disk_wakeup: Option<ObjectId>, // Ref to diskWakeup
    #[serde(rename="usedMemory")]
	pub used_memory: u64, // Used memory by the user
    #[serde(rename="createdAt")]
	pub created_at: DateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproxUserDiskInfo {
	#[serde(rename = "diskId", skip_serializing_if = "Option::is_none")]
	pub disk_id: Option<ObjectId>,
	#[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
	pub user_id: Option<ObjectId>
}

#[derive(Debug)]
pub enum DiskAction { // todo protobuf with enum so that it is compatible with any other service
    READ,
    WRITE,
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
	pub disk_id: Option<ObjectId>,
	pub user_id: Option<ObjectId>,
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
	pub startup: Option<Event>, // todo error with that : data retrieved by find_one doesn't seem to fit inside of that
    // #[serde(rename = "periodInfo")]
	// period_info: PeriodInfo,
	pub shutdown: Option<Event>, // null until disk shutdown // todo or error with that
    #[serde(rename = "periodInfo")]
	pub period_info: Option<PeriodInfo> // Disk consumption since disk startup, null until disk shutdown
}

#[derive(Debug, Clone)]
pub struct AvailableDisk {
	pub uid: String,
	/// type either HDD, SSD or unknown
    pub type_: DiskKind,
	/// device_name as string
    pub device_name: String,
    // file_system: Vec<u8>,
	/// mount_point as dirpath
    pub mount_point: String,
	/// total_space in bytes
    pub total_space: u64,
	/// available_space in bytes
    pub available_space: u64
    // is_removable: bool,
}