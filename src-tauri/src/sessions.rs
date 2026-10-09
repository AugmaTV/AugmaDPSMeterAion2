use std::cmp::Reverse;
use std::fs;
use std::io::{Error, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::meter::{summarize, Fight, Snapshot, Status};

const DIRECTORY: &str = "sessions";
const INDEX: &str = "index.json";
const KEPT: usize = 200;

static LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
	pub id: u64,
	pub name: Option<String>,
	pub boss: Option<u32>,
	pub start: u64,
	pub duration: u64,
	pub fights: usize,
	pub dps: u64,
	#[serde(default)]
	pub locked: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
	pub fights: Vec<Overview>,
	pub snapshot: Snapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
	pub boss: Option<u32>,
	pub duration: u64,
}

pub fn directory(app: &AppHandle) -> Result<PathBuf> {
	app.path().app_data_dir().map(|directory| directory.join(DIRECTORY)).map_err(Error::other)
}

pub fn save(directory: &Path, fights: &[Fight]) -> Result<()> {
	let Some(first) = fights.first() else {
		return Ok(());
	};
	let _lock = LOCK.lock();
	fs::create_dir_all(directory)?;
	fs::write(file(directory, first.start), serde_json::to_vec(fights)?)?;
	let snapshot = summarize(Status::Waiting, &fights.iter().collect::<Vec<_>>(), true);
	let mut index = index(directory);
	let previous = index.iter().position(|summary| summary.id == first.start).map(|position| index.remove(position));
	index.push(Summary {
		id: first.start,
		name: previous.as_ref().and_then(|summary| summary.name.clone()),
		boss: fights.iter().filter_map(|fight| fight.boss.as_ref()).max_by_key(|boss| boss.max).and_then(|boss| boss.npc),
		start: first.start / 1000,
		duration: snapshot.duration,
		fights: fights.len(),
		dps: snapshot.players.iter().map(|player| player.dps).sum(),
		locked: previous.is_some_and(|summary| summary.locked),
	});
	index.sort_by_key(|summary| Reverse(summary.id));
	let expired: Vec<u64> = index.iter().skip(KEPT).filter(|summary| !summary.locked).map(|summary| summary.id).collect();
	for id in &expired {
		let _ = fs::remove_file(file(directory, *id));
	}
	index.retain(|summary| !expired.contains(&summary.id));
	write(directory, &index)
}

pub fn list(directory: &Path) -> Vec<Summary> {
	let _lock = LOCK.lock();
	index(directory)
}

pub fn view(directory: &Path, id: u64, fight: Option<usize>, party_only: bool) -> Option<View> {
	let fights: Vec<Fight> = serde_json::from_slice(&fs::read(file(directory, id)).ok()?).ok()?;
	let selected: Vec<&Fight> = match fight {
		Some(fight) => fights.get(fight).into_iter().collect(),
		None => fights.iter().collect(),
	};
	Some(View {
		fights: fights.iter().map(|fight| Overview { boss: fight.boss.as_ref().and_then(|boss| boss.npc), duration: (fight.last - fight.start) / 1000 }).collect(),
		snapshot: summarize(Status::Waiting, &selected, party_only),
	})
}

pub fn rename(directory: &Path, id: u64, name: Option<String>) -> Result<()> {
	update(directory, id, |summary| summary.name = name)
}

pub fn lock(directory: &Path, id: u64, locked: bool) -> Result<()> {
	update(directory, id, |summary| summary.locked = locked)
}

pub fn delete(directory: &Path, id: u64) -> Result<()> {
	let _lock = LOCK.lock();
	let mut index = index(directory);
	index.retain(|summary| summary.id != id);
	let _ = fs::remove_file(file(directory, id));
	write(directory, &index)
}

fn update(directory: &Path, id: u64, change: impl FnOnce(&mut Summary)) -> Result<()> {
	let _lock = LOCK.lock();
	let mut index = index(directory);
	if let Some(summary) = index.iter_mut().find(|summary| summary.id == id) {
		change(summary);
	}
	write(directory, &index)
}

fn file(directory: &Path, id: u64) -> PathBuf {
	directory.join(format!("{id}.json"))
}

fn index(directory: &Path) -> Vec<Summary> {
	fs::read(directory.join(INDEX)).ok().and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default()
}

fn write(directory: &Path, index: &[Summary]) -> Result<()> {
	fs::write(directory.join(INDEX), serde_json::to_vec(index)?)
}

#[cfg(test)]
mod tests {
	use std::env;
	use std::process;

	use super::*;
	use crate::meter::Meter;
	use crate::packet::{Event, Hit};

	fn fights(start: u64) -> Vec<Fight> {
		let mut meter = Meter::default();
		meter.apply(start, Event::Character { entity: 1, name: String::from("Moi"), own: true, equipment: Vec::new() });
		meter.apply(start, Event::Hit(Hit { target: 900, actor: 1, skill: 11020000, damage: 1000, dot: false, drain: 0, ..Hit::default() }));
		meter.apply(start + 2_000_000, Event::Hit(Hit { target: 900, actor: 1, skill: 11020000, damage: 1000, dot: false, drain: 0, ..Hit::default() }));
		meter.session()
	}

	fn temporary(name: &str) -> PathBuf {
		let directory = env::temp_dir().join(format!("augma-sessions-{name}-{}", process::id()));
		let _ = fs::remove_dir_all(&directory);
		directory
	}

	#[test]
	fn saves_renames_and_deletes_sessions() {
		let directory = temporary("manage");
		save(&directory, &fights(1_000_000)).unwrap();
		rename(&directory, 1_000_000, Some(String::from("Run"))).unwrap();
		lock(&directory, 1_000_000, true).unwrap();
		save(&directory, &fights(1_000_000)).unwrap();
		let sessions = list(&directory);
		assert_eq!((sessions.len(), sessions[0].name.as_deref(), sessions[0].locked, sessions[0].duration, sessions[0].dps), (1, Some("Run"), true, 2000, 1000));
		assert_eq!(view(&directory, 1_000_000, None, false).unwrap().snapshot.total, 2000);
		delete(&directory, 1_000_000).unwrap();
		assert!(list(&directory).is_empty() && view(&directory, 1_000_000, None, false).is_none());
		let _ = fs::remove_dir_all(&directory);
	}

	#[test]
	fn keeps_recent_and_locked_sessions() {
		let directory = temporary("prune");
		save(&directory, &fights(1_000_000)).unwrap();
		lock(&directory, 1_000_000, true).unwrap();
		save(&directory, &fights(2_000_000)).unwrap();
		rename(&directory, 2_000_000, Some(String::from("Renommée"))).unwrap();
		for index in 3..=(KEPT as u64 + 2) {
			save(&directory, &fights(index * 1_000_000)).unwrap();
		}
		let sessions = list(&directory);
		assert_eq!(sessions.len(), KEPT + 1);
		assert!(sessions.iter().any(|summary| summary.id == 1_000_000) && !sessions.iter().any(|summary| summary.id == 2_000_000));
		assert!(!file(&directory, 2_000_000).exists());
		let _ = fs::remove_dir_all(&directory);
	}

	#[test]
	fn reads_index_without_lock() {
		let directory = temporary("legacy");
		fs::create_dir_all(&directory).unwrap();
		fs::write(directory.join(INDEX), r#"[{"id":1,"name":null,"boss":null,"start":0,"duration":0,"fights":1,"dps":0}]"#).unwrap();
		assert!(list(&directory).iter().all(|summary| !summary.locked) && list(&directory).len() == 1);
		let _ = fs::remove_dir_all(&directory);
	}
}