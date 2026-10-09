use std::fs;
use std::io::{Error, Result};
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::meter::{Profile, Profiles};

const FILE: &str = "profiles.json";

static LOCK: Mutex<()> = Mutex::new(());

pub fn load(app: &AppHandle) -> Profiles {
	app.path().app_data_dir().ok().and_then(|directory| fs::read(directory.join(FILE)).ok()).and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default()
}

pub fn last(app: &AppHandle) -> Option<Profile> {
	let profiles = load(app);
	profiles.last.as_ref().and_then(|name| profiles.characters.get(name)).cloned()
}

pub fn save(app: &AppHandle, profiles: &Profiles) -> Result<()> {
	let _lock = LOCK.lock();
	let directory = app.path().app_data_dir().map_err(Error::other)?;
	fs::create_dir_all(&directory)?;
	fs::write(directory.join(FILE), serde_json::to_vec(profiles)?)
}