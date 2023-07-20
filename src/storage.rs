use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Instant;
use std::fs::{self, OpenOptions};
use std::io::{prelude::*, BufReader};

use anyhow::{ensure, Result};
use log::{debug, info};

use crate::location::Location;

pub struct Storage {
    base_path: PathBuf,

    users: HashMap<String, UserStorage>,
}

pub struct UserStorage {
    devices: HashMap<String, DeviceStorage>,
}

pub struct DeviceStorage {
    locations: Vec<Location>,
}

impl UserStorage {
    pub fn devices(&self) -> &HashMap<String, DeviceStorage> { &self.devices }
    pub fn device_names(&self) -> Vec<String> { self.devices.keys().cloned().collect() }

    pub fn device(&self, device_name: &str) -> Option<&DeviceStorage> { self.devices.get(device_name) }
}

impl DeviceStorage {
    pub fn locations(&self) -> &Vec<Location> { &self.locations }
    pub fn last_location(&self) -> Option<&Location> { self.locations.last() }
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Storage {
        Storage {
            base_path,

            users: HashMap::new(),
        }
    }

    pub fn users(&self) -> &HashMap<String, UserStorage> { &self.users }
    pub fn user_names(&self) -> Vec<String> { self.users.keys().cloned().collect() }

    pub fn user(&self, user_name: &str) -> Option<&UserStorage> { self.users.get(user_name) }

    pub fn read_from_fs(&mut self) -> Result<()> {
        let base_path = &self.base_path;
        let base_path_str = base_path.to_str().unwrap_or("None");

        info!(target: "Storage", "Loading from base directory '{}'", base_path_str);

        let started_loading = Instant::now();

        let base_path_metadata = base_path.metadata();
        if base_path_metadata.is_err() {
            debug!(target: "Storage", "No base directory at '{}', creating", base_path_str);
            fs::create_dir_all(&base_path)?;
        }

        ensure!(base_path_metadata.unwrap().is_dir(), format!("Base path '{}' does not point to a directory", base_path_str));

        let last_directory = base_path.join(PathBuf::from("last"));
        let history_directory = base_path.join(PathBuf::from("rec"));

        // NOTE: The last directory is used as the source of truth for which users and devices exist.
        //       The history directory will be traversed according to the users and devices discovered here.
        for last_directory_for_user in last_directory.read_dir()? {
            let last_directory_for_user = last_directory_for_user?.path();
            let user_name = last_directory_for_user.file_name().map(OsStr::to_str).flatten().unwrap_or("None");

            self.users.insert(user_name.to_string(), UserStorage { devices: HashMap::new() });

            for last_directory_for_user_and_device in last_directory_for_user.read_dir()? {
                let last_directory_for_user_and_device = last_directory_for_user_and_device?.path();
                let device_name = last_directory_for_user_and_device.file_name().map(OsStr::to_str).flatten().unwrap_or("None");

                let last_file_for_user_and_device = last_directory_for_user_and_device
                    .join(PathBuf::from("{user_name}-{device_name}.json"));

                let device_storage = DeviceStorage {
                    locations: Vec::new(),
                };

                self.users.get_mut(user_name).unwrap().devices.insert(device_name.to_string(), device_storage);

                // NOTE: We don't actually read the file at storage-directory/last/{user}/{device}/{user}-{device}.json,
                //       the in memory representation of the location history is entirely loaded from the history
                //       directory. We do however write out an updated latest file with each sync.
            }
        }

        for (user_name, user_storage) in &mut self.users {
            for (device_name, device_storage) in &mut user_storage.devices {
                let history_directory_for_user_and_device = history_directory.join(user_name).join(device_name);

                for history_for_user_and_device_in_month in history_directory_for_user_and_device.read_dir()? {
                    let history_file = OpenOptions::new()
                        .append(true)
                        .read(true)
                        .open(history_for_user_and_device_in_month?.path())?;

                    for line in BufReader::new(&history_file).lines() {
                        let line = line?;

                        // Same as location.timestamp apparently.
                        let _: String = line.chars().take_while(|char| char != &'\t').collect();
                        let json: String = line.chars().skip_while(|char| char != &'{').collect();

                        let location: Location = serde_json::from_str(&json)?;

                        device_storage.locations.push(location);
                    }
                }
            }
        }

        info!(target: "Storage", "Loading took {:.2?}, loaded {} user(s) with a total of {} location(s)", started_loading.elapsed(), self.users.len(), self.users.iter()
            .flat_map(|(_, user_storage)| &user_storage.devices)
            .map(|(_, device_storage)| &device_storage.locations)
            .map(|locations| locations.len())
            .fold(0, |acc, len| acc + len));

        let started_sorting = Instant::now();

        for device_storage in self.users.iter_mut()
            .flat_map(|(_, user_storage)| &mut user_storage.devices)
            .map(|(_, device_storage)| device_storage)
        {
            device_storage.locations.sort_by(|x, y| x.timestamp.cmp(&y.timestamp));
        }

        info!(target: "Storage", "Sorting locations took {:.2?}", started_sorting.elapsed());

        Ok(())
    }
}
