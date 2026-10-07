use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use serde::Serialize;

use crate::packet::{Event, Hit};

const IDLE_MICROS: u64 = 15_000_000;
const CLASS_SKILLS: Range<u32> = 11_000_000..20_000_000;
const CLASS_COUNT: usize = 10;

#[derive(Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
	Unavailable,
	Waiting,
	Live,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
	pub status: Status,
	pub duration: u64,
	pub total: u64,
	pub players: Vec<Player>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Player {
	pub id: u64,
	pub name: Option<String>,
	pub class: u8,
	pub damage: u64,
	pub dps: u64,
	pub hits: u64,
	pub crits: u64,
	pub own: bool,
}

#[derive(Default)]
pub struct Meter {
	names: HashMap<u64, String>,
	classes: HashMap<u64, [u32; CLASS_COUNT]>,
	spawned: HashSet<u64>,
	owners: HashMap<u64, u64>,
	own: Option<u64>,
	encounter: Option<Encounter>,
}

struct Encounter {
	start: u64,
	last: u64,
	ended: bool,
	actors: HashMap<u64, Stats>,
}

#[derive(Default)]
struct Stats {
	damage: u64,
	hits: u64,
	crits: u64,
}

impl Meter {
	pub fn apply(&mut self, micros: u64, event: Event) {
		match event {
			Event::Hit(hit) => self.hit(micros, hit),
			Event::Character { entity, name, own } => {
				if own {
					self.own = Some(entity);
				}
				self.spawned.remove(&entity);
				self.owners.remove(&entity);
				self.names.insert(entity, name);
			}
			Event::Spawn { entity, owner } => {
				self.names.remove(&entity);
				self.classes.remove(&entity);
				self.spawned.insert(entity);
				match owner {
					Some(owner) => self.owners.insert(entity, owner),
					None => self.owners.remove(&entity),
				};
			}
			Event::MapChange => {
				if let Some(encounter) = &mut self.encounter {
					encounter.ended = true;
				}
			}
		}
	}

	pub fn reset(&mut self) {
		self.encounter = None;
	}

	pub fn snapshot(&self, status: Status) -> Snapshot {
		let Some(encounter) = &self.encounter else {
			return Snapshot { status, duration: 0, total: 0, players: Vec::new() };
		};
		let duration = encounter.last - encounter.start;
		let seconds = (duration as f64 / 1_000_000.0).max(1.0);
		let mut players: Vec<Player> = encounter
			.actors
			.iter()
			.filter(|(id, _)| self.is_player(**id))
			.map(|(id, stats)| Player {
				id: *id,
				name: self.names.get(id).cloned(),
				class: self.class_of(*id),
				damage: stats.damage,
				dps: (stats.damage as f64 / seconds) as u64,
				hits: stats.hits,
				crits: stats.crits,
				own: self.own == Some(*id),
			})
			.collect();
		players.sort_by_key(|player| Reverse(player.damage));
		Snapshot { status, duration: duration / 1000, total: players.iter().map(|player| player.damage).sum(), players }
	}

	fn hit(&mut self, micros: u64, hit: Hit) {
		if !hit.dot && CLASS_SKILLS.contains(&hit.skill) && !self.spawned.contains(&hit.actor) {
			let votes = self.classes.entry(hit.actor).or_default();
			if hit.actor != hit.target {
				votes[(hit.skill / 1_000_000 - 10) as usize] += 1;
			}
		}
		let actor = self.owners.get(&hit.actor).copied().unwrap_or(hit.actor);
		if self.is_player(hit.target) {
			return;
		}
		let starts = self.is_player(actor);
		let active = self.encounter.as_ref().is_some_and(|encounter| !encounter.ended && micros.saturating_sub(encounter.last) < IDLE_MICROS);
		if !active {
			if !starts {
				return;
			}
			self.encounter = None;
		}
		let encounter = self.encounter.get_or_insert_with(|| Encounter { start: micros, last: micros, ended: false, actors: HashMap::new() });
		if starts {
			encounter.last = encounter.last.max(micros);
		}
		let stats = encounter.actors.entry(actor).or_default();
		stats.damage += hit.damage;
		if !hit.dot {
			stats.hits += 1;
			stats.crits += hit.critical as u64;
		}
	}

	fn is_player(&self, entity: u64) -> bool {
		!self.spawned.contains(&entity) && (self.classes.contains_key(&entity) || self.names.contains_key(&entity))
	}

	fn class_of(&self, entity: u64) -> u8 {
		self.classes.get(&entity).and_then(|votes| (1..CLASS_COUNT).filter(|class| votes[*class] > 0).max_by_key(|class| votes[*class])).unwrap_or_default() as u8
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn hit(actor: u64, skill: u32, damage: u64) -> Event {
		Event::Hit(Hit { target: 900, actor, skill, damage, critical: false, dot: false })
	}

	#[test]
	fn aggregates_player_damage_and_summons() {
		let mut meter = Meter::default();
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, Event::Spawn { entity: 50, owner: Some(2) });
		meter.apply(0, Event::Spawn { entity: 51, owner: None });
		meter.apply(1_000_000, hit(1, 14010353, 1000));
		meter.apply(1_000_000, hit(2, 16000010, 500));
		meter.apply(2_000_000, hit(50, 16250001, 250));
		meter.apply(2_000_000, hit(51, 16250001, 300));
		meter.apply(3_000_000, hit(77, 1230790, 9999));
		let snapshot = meter.snapshot(Status::Live);
		assert_eq!(snapshot.total, 1750);
		assert_eq!(snapshot.players.len(), 2);
		assert_eq!(snapshot.players[0].name.as_deref(), Some("Moi"));
		assert_eq!(snapshot.players[0].class, 4);
		assert_eq!(snapshot.players[1].damage, 750);
	}

	#[test]
	fn keeps_majority_class() {
		let mut meter = Meter::default();
		meter.apply(0, hit(1, 16000010, 10));
		meter.apply(0, hit(1, 16000020, 10));
		meter.apply(0, hit(1, 18160032, 10));
		assert_eq!(meter.snapshot(Status::Live).players[0].class, 6);
	}

	#[test]
	fn starts_new_encounter_after_idle() {
		let mut meter = Meter::default();
		meter.apply(0, hit(1, 11020000, 100));
		meter.apply(IDLE_MICROS + 1, hit(1, 11020000, 40));
		assert_eq!(meter.snapshot(Status::Live).total, 40);
	}
}