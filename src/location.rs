use chrono::{DateTime, Utc, serde::{ts_seconds, ts_seconds_option}};

use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

// 0=unknown, 1=unplugged, 2=charging, 3=full
#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u8)]
pub enum BatteryStatus {
    Unknown = 0,
    Unplugged = 1,
    Charging = 2,
    Full = 3,
}

// significant=1, move=2
#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(i8)]
pub enum MonitoringMode {
    Quiet = -1,
    Manual = 0,
    Significant = 1,
    Move = 2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "conn")]
pub enum Connection {
    // phone is connected to a WiFi connection
    // (iOS,Android)
    #[serde(rename = "w")]
    Wifi {
        #[serde(flatten)]
        metadata: Option<WifiMetadata>,
    },

    // phone is offline
    // (iOS,Android)
    #[serde(rename = "o")]
    Offline,

    // mobile data
    // (iOS,Android)
    #[serde(rename = "m")]
    Mobile,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WifiMetadata {
    // if available, is the unique name of the WLAN.
    // (iOS,string/optional)
    #[serde(rename = "SSID")]
    pub ssid: String,

    // if available, identifies the access point.
    // (iOS,string/optional)
    #[serde(rename = "BSSID")]
    pub bssid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Trigger {
    // ping issued randomly by background task
    #[serde(rename = "p")]
    Ping,

    // circular region enter/leave event
    #[serde(rename = "c")]
    CircularRegion,

    // beacon region enter/leave event (iOS only)
    #[serde(rename = "b")]
    BeaconRegion,

    // response to a reportLocation cmd message
    #[serde(rename = "r")]
    ReportLocationResponse,

    // manual publish requested by the user
    #[serde(rename = "u")] 
    Manual,

    // timer based publish in move move (iOS only)
    #[serde(rename = "t")]
    Timer,

    // updated by Settings/Privacy/Locations Services/System Services/Frequent Locations monitoring (iOS only)
    #[serde(rename = "v")]
    LocationsServices,
}

// See https://owntracks.org/booklet/tech/json/#_typelocation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Location {
    // Accuracy of the reported location in meters without unit
    // (iOS,Android/integer/meters/optional)
    #[serde(rename = "acc")]
    pub accuracy: Option<u32>,

    // Altitude measured above sea level
    // (iOS,Android/integer/meters/optional)
    #[serde(rename = "alt")]
    pub altitude: Option<i32>,

    // Device battery level
    // (iOS,Android/integer/percent/optional)
    #[serde(rename = "batt")]
    pub battery: Option<u8>,

    // Battery Status
    // (iOS, Android)
    #[serde(rename = "bs")]
    pub battery_status: BatteryStatus,

    // Course over ground
    // (iOS/integer/degree/optional)
    #[serde(rename = "cog")]
    pub course: Option<u16>,

    // latitude
    // (iOS,Android/float/degree/required)
    #[serde(rename = "lat")]
    pub latitude: f32,

    // longitude
    // (iOS,Android/float/degree/required)
    #[serde(rename = "lon")]
    pub longitude: f32,

    // radius around the region when entering/leaving
    // (iOS/integer/meters/optional)
    #[serde(rename = "rad")]
    pub region_radius: Option<u32>,

    // trigger for the location report
    // (iOS,Android/string/optional)
    #[serde(rename = "t")]
    pub trigger: Option<Trigger>,

    // Tracker ID used to display the initials of a user
    // (iOS,Android/string/optional) required for http mode
    #[serde(rename = "tid")]
    pub tracker_id: Option<String>,

    // UNIX epoch timestamp in seconds of the location fix
    // (iOS,Android/integer/epoch/required)
    #[serde(rename = "tst", with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,

    // vertical accuracy of the alt element
    // (iOS/integer/meters/optional)
    #[serde(rename = "vac")]
    pub vertical_accuracy: Option<u32>,

    // velocity
    // (iOS,Android/integer/kmh/optional)
    #[serde(rename = "vel")]
    pub velocity: Option<u32>,

    // barometric pressure
    // (iOS/float/kPa/optional/extended data)
    #[serde(rename = "p")]
    pub barometric_pressure: Option<f32>,

    // point of interest name
    // (iOS/string/optional)
    #[serde(rename = "poi")]
    pub point_of_interest_name: Option<String>,

    // Internet connectivity status (route to host) when the message is created
    // (iOS,Android/string/optional/extended data)
    #[serde(flatten)]
    pub connection: Option<Connection>,

    // name of the tag
    // (iOS/string/optional)
    pub tag: Option<String>,

    // (only in HTTP payloads) contains the original publish topic (e.g. owntracks/jane/phone).
    // (iOS,Android >= 2.4,string)
    pub topic: Option<String>,

    // contains a list of regions the device is currently in (e.g. ["Home","Garage"]). Might be empty.
    // (iOS,Android/list of strings/optional)
    #[serde(rename = "inregions")]
    pub in_regions: Option<Vec<String>>,

    // contains a list of region IDs the device is currently in (e.g. ["6da9cf","3defa7"]). Might be empty.
    // (iOS,Android/list of strings/optional)
    #[serde(rename = "inrids")]
    pub in_region_ids: Option<Vec<String>>,

    // identifies the time at which the message is constructed (vs. tst which is the timestamp of the GPS fix)
    // (iOS,Android)
    // NOTE: Even though this is not documented as such, this field appears to be optional (at least
    //       given my location history created with the iOS client).
    #[serde(with = "ts_seconds_option", default)]
    pub created_at: Option<DateTime<Utc>>,

    // identifies the monitoring mode at which the message is constructed
    // (iOS/integer/optional)
    #[serde(rename = "m")]
    pub monitoring_mode: Option<MonitoringMode>,
}
