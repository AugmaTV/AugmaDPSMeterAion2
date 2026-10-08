use std::fs;
use std::io::{Error, Result};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const FILE: &str = "settings.json";

static LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
	pub party_only: bool,
	pub dungeon: bool,
	pub locked: bool,
	pub overlays: Vec<String>,
}

impl Default for Settings {
	fn default() -> Self {
		Settings { party_only: true, dungeon: false, locked: false, overlays: Vec::new() }
	}
}

pub fn load(app: &AppHandle) -> Settings {
	app.path().app_data_dir().ok().and_then(|directory| fs::read(directory.join(FILE)).ok()).and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<()> {
	let _lock = LOCK.lock();
	let directory = app.path().app_data_dir().map_err(Error::other)?;
	fs::create_dir_all(&directory)?;
	fs::write(directory.join(FILE), serde_json::to_vec(settings)?)
}