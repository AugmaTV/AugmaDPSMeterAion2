use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use serde::Serialize;

use crate::packet::{family, Event, Heal, Hit, Vitals as SpawnVitals};

const IDLE_MICROS: u64 = 15_000_000;
const GRACE_MICROS: u64 = 2_000_000;
const BOSS_HP: u64 = 1_000_000;
const CLASS_SKILLS: Range<u32> = 11_000_000..20_000_000;
const SPIRIT_SKILLS: [Range<u32>; 2] = [16_000_000..16_010_000, 16_990_000..17_000_000];
const MONSTER_SKILLS: Range<u32> = 1_000_000..2_000_000;
const CLASS_COUNT: usize = 10;
const DRAIN_SKILL: u32 = 1;

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
	pub total_healing: u64,
	pub total_taken: u64,
	pub boss: Option<Boss>,
	pub players: Vec<Player>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Boss {
	pub npc: Option<u32>,
	pub hp: u64,
	pub max: u64,
	pub estimated: bool,
	pub dead: bool,
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
	pub healing: u64,
	pub hps: u64,
	pub taken: u64,
	pub dtps: u64,
	pub taken_hits: u64,
	pub own: bool,
	#[serde(skip)]
	pub member: bool,
	pub skills: Vec<Skill>,
	pub heals: Vec<Skill>,
	pub sources: Vec<Skill>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
	pub id: u32,
	pub amount: u64,
	pub hits: u64,
	pub crits: u64,
}

#[derive(Default)]
pub struct Meter {
	names: HashMap<u64, String>,
	classes: HashMap<u64, [u32; CLASS_COUNT]>,
	spawned: HashSet<u64>,
	owners: HashMap<u64, u64>,
	vitals: HashMap<u64, Vitals>,
	own: Option<u64>,
	party: HashSet<u64>,
	roster: HashSet<u64>,
	party_only: bool,
	map: Option<u32>,
	encounter: Option<Encounter>,
}

#[derive(Default)]
struct Vitals {
	npc: Option<u32>,
	hp: u64,
	max: Option<u64>,
	highest: u64,
	first: Option<u64>,
	before: u64,
	in_combat: bool,
	combat_seen: bool,
	dead: bool,
}

struct Encounter {
	start: u64,
	last: u64,
	ended: Option<u64>,
	targets: HashMap<u64, u64>,
	actors: HashMap<u64, Stats>,
	members: HashSet<u64>,
	frozen: Option<(Vec<Player>, Option<Boss>)>,
}

#[derive(Default)]
struct Stats {
	damage: u64,
	hits: u64,
	crits: u64,
	healing: u64,
	taken: u64,
	taken_hits: u64,
	skills: HashMap<u32, Totals>,
	heals: HashMap<u32, Totals>,
	sources: HashMap<u32, Totals>,
}

#[derive(Default)]
struct Totals {
	amount: u64,
	hits: u64,
	crits: u64,
}

impl Meter {
	pub fn apply(&mut self, micros: u64, event: Event) {
		match event {
			Event::Hit(hit) => self.hit(micros, hit),
			Event::Heal(heal) => self.heal(micros, heal),
			Event::Character { entity, name, own } => {
				if own {
					self.own = Some(entity);
				}
				self.spawned.remove(&entity);
				self.owners.remove(&entity);
				self.names.insert(entity, name);
			}
			Event::Spawn { entity, owner, vitals } => self.spawn(entity, owner, vitals),
			Event::Health { entity, hp } => self.health(micros, entity, hp),
			Event::Despawn { entity, dead } => self.despawn(micros, entity, dead),
			Event::Combat { entity, active } => self.combat(micros, entity, active),
			Event::MapChange { map, revive } => {
				if self.map != Some(map) || (!revive && !self.boss_in_combat(None)) {
					self.end(micros);
				}
				self.map = Some(map);
			}
			Event::PartyMember { entity } => self.join(entity),
			Event::PartyRoster { entities } => {
				for left in self.roster.difference(&entities) {
					self.party.remove(left);
				}
				for entity in &entities {
					self.join(*entity);
				}
				self.roster = entities;
			}
		}
	}

	pub fn reconnect(&mut self, micros: u64) {
		self.end(micros);
		let frozen = self.encounter.as_ref().map(|encounter| (self.players(encounter), self.boss(encounter)));
		if let (Some(encounter), Some(frozen)) = (&mut self.encounter, frozen) {
			encounter.frozen.get_or_insert(frozen);
		}
		self.names.clear();
		self.classes.clear();
		self.spawned.clear();
		self.owners.clear();
		self.vitals.clear();
		self.own = None;
		self.party.clear();
		self.roster.clear();
	}

	pub fn reset(&mut self) {
		self.encounter = None;
	}

	pub fn set_party_only(&mut self, enabled: bool) {
		self.party_only = enabled;
	}

	pub fn snapshot(&self, status: Status) -> Snapshot {
		let Some(encounter) = &self.encounter else {
			return Snapshot { status, duration: 0, total: 0, total_healing: 0, total_taken: 0, boss: None, players: Vec::new() };
		};
		let (mut players, boss) = match &encounter.frozen {
			Some((players, boss)) => (players.clone(), boss.clone()),
			None => (self.players(encounter), self.boss(encounter)),
		};
		if self.party_only {
			players.retain(|player| player.member);
		}
		Snapshot {
			status,
			duration: (encounter.last - encounter.start) / 1000,
			total: players.iter().map(|player| player.damage).sum(),
			total_healing: players.iter().map(|player| player.healing).sum(),
			total_taken: players.iter().map(|player| player.taken).sum(),
			boss,
			players,
		}
	}

	fn hit(&mut self, micros: u64, hit: Hit) {
		if hit.actor == hit.target {
			return;
		}
		if !hit.dot {
			self.vote(hit.actor, hit.target, hit.skill);
		}
		let actor = self.owner(hit.actor);
		if self.is_player(hit.target) {
			if !self.is_player(actor) {
				self.take(micros, hit);
			}
			return;
		}
		if self.owner(hit.target) == actor {
			return;
		}
		let starts = self.is_player(actor) && (!self.party_only || self.member(actor));
		let dead = self.vitals.get(&hit.target).is_some_and(|vitals| vitals.dead);
		if !self.active(micros) {
			let late = dead && self.encounter.as_ref().is_some_and(|encounter| encounter.ended.is_some_and(|ended| micros.saturating_sub(ended) < GRACE_MICROS) && encounter.targets.contains_key(&hit.target));
			if !late {
				if dead || !starts {
					return;
				}
				self.encounter = None;
			}
		} else if dead && !self.encounter.as_ref().is_some_and(|encounter| encounter.targets.contains_key(&hit.target)) {
			return;
		}
		if starts {
			let vitals = self.vitals.entry(hit.target).or_default();
			if vitals.first.is_none() {
				vitals.before += hit.damage;
			}
		}
		let encounter = self.encounter.get_or_insert_with(|| Encounter { start: micros, last: micros, ended: None, targets: HashMap::new(), actors: HashMap::new(), members: self.party.clone(), frozen: None });
		if starts {
			encounter.last = encounter.last.max(micros);
			*encounter.targets.entry(hit.target).or_default() += hit.damage;
		}
		let stats = encounter.actors.entry(actor).or_default();
		let skill = stats.skills.entry(base_skill(hit.skill)).or_default();
		stats.damage += hit.damage;
		skill.amount += hit.damage;
		if !hit.dot {
			stats.hits += 1;
			stats.crits += hit.critical as u64;
			skill.hits += 1;
			skill.crits += hit.critical as u64;
		}
		if hit.drain > 0 && starts {
			stats.healing += hit.drain;
			let drain = stats.heals.entry(DRAIN_SKILL).or_default();
			drain.amount += hit.drain;
			drain.hits += 1;
		}
	}

	fn heal(&mut self, micros: u64, heal: Heal) {
		if self.spawned.contains(&heal.target) {
			return;
		}
		self.vote(heal.actor, heal.target, heal.skill);
		let healer = self.owner(heal.actor);
		if !self.is_player(healer) || !self.active(micros) {
			return;
		}
		let Some(encounter) = &mut self.encounter else {
			return;
		};
		let stats = encounter.actors.entry(healer).or_default();
		let skill = stats.heals.entry(base_skill(heal.skill)).or_default();
		stats.healing += heal.amount;
		skill.amount += heal.amount;
		skill.hits += 1;
	}

	fn take(&mut self, micros: u64, hit: Hit) {
		if !self.active(micros) || (hit.dot && !MONSTER_SKILLS.contains(&hit.skill)) {
			return;
		}
		let npc = self.vitals.get(&hit.actor).and_then(|vitals| vitals.npc).unwrap_or_default();
		let Some(encounter) = &mut self.encounter else {
			return;
		};
		let stats = encounter.actors.entry(hit.target).or_default();
		let source = stats.sources.entry(npc).or_default();
		stats.taken += hit.damage;
		source.amount += hit.damage;
		if !hit.dot {
			stats.taken_hits += 1;
			source.hits += 1;
		}
	}

	fn spawn(&mut self, entity: u64, owner: Option<u64>, vitals: Option<SpawnVitals>) {
		self.names.remove(&entity);
		self.classes.remove(&entity);
		self.spawned.insert(entity);
		match owner {
			Some(owner) => self.owners.insert(entity, owner),
			None => self.owners.remove(&entity),
		};
		if let Some(vitals) = vitals {
			self.vitals.insert(entity, Vitals { npc: Some(vitals.npc), hp: vitals.hp, max: Some(vitals.max), highest: vitals.hp, first: Some(vitals.hp), dead: vitals.hp == 0, ..Vitals::default() });
		}
	}

	fn health(&mut self, micros: u64, entity: u64, hp: u64) {
		let vitals = self.vitals.entry(entity).or_default();
		vitals.hp = hp;
		vitals.highest = vitals.highest.max(hp);
		vitals.first.get_or_insert(hp);
		vitals.dead = hp == 0;
		if hp == 0 {
			vitals.in_combat = false;
		}
		let reset = vitals.max == Some(hp) && vitals.combat_seen && !vitals.in_combat;
		if hp == 0 || reset {
			self.finish(micros, entity);
		}
	}

	fn despawn(&mut self, micros: u64, entity: u64, dead: bool) {
		let Some(vitals) = self.vitals.get_mut(&entity) else {
			return;
		};
		if dead {
			vitals.hp = 0;
			vitals.dead = true;
			self.finish(micros, entity);
		} else {
			vitals.in_combat = false;
		}
	}

	fn combat(&mut self, micros: u64, entity: u64, active: bool) {
		let vitals = self.vitals.entry(entity).or_default();
		vitals.combat_seen = true;
		if vitals.dead && active {
			return;
		}
		vitals.in_combat = active;
		if !active && (vitals.hp == 0 || vitals.max == Some(vitals.hp)) {
			self.finish(micros, entity);
		}
	}

	fn join(&mut self, entity: u64) {
		self.party.insert(entity);
		if let Some(encounter) = &mut self.encounter {
			encounter.members.insert(entity);
		}
	}

	fn finish(&mut self, micros: u64, entity: u64) {
		let boss = self.vitals.get(&entity).is_some_and(|vitals| vitals.limit() >= BOSS_HP);
		let tracked = self.encounter.as_ref().is_some_and(|encounter| encounter.targets.contains_key(&entity));
		if boss && tracked && !self.boss_in_combat(Some(entity)) {
			self.end(micros);
		}
	}

	fn end(&mut self, micros: u64) {
		if let Some(encounter) = &mut self.encounter {
			encounter.ended.get_or_insert(micros);
		}
	}

	fn active(&self, micros: u64) -> bool {
		self.encounter.as_ref().is_some_and(|encounter| encounter.ended.is_none() && (micros.saturating_sub(encounter.last) < IDLE_MICROS || self.boss_in_combat(None)))
	}

	fn boss_in_combat(&self, excluded: Option<u64>) -> bool {
		self.encounter.as_ref().is_some_and(|encounter| {
			encounter.targets.keys().filter(|target| Some(**target) != excluded).any(|target| self.vitals.get(target).is_some_and(|vitals| vitals.limit() >= BOSS_HP && vitals.in_combat && !vitals.dead))
		})
	}

	fn players(&self, encounter: &Encounter) -> Vec<Player> {
		let seconds = ((encounter.last - encounter.start) as f64 / 1_000_000.0).max(1.0);
		let mut players: Vec<Player> = encounter
			.actors
			.iter()
			.filter(|(id, stats)| self.is_player(**id) && (stats.damage > 0 || stats.healing > 0 || stats.taken > 0))
			.map(|(id, stats)| Player {
				id: *id,
				name: self.names.get(id).cloned(),
				class: self.class_of(*id),
				damage: stats.damage,
				dps: (stats.damage as f64 / seconds) as u64,
				hits: stats.hits,
				crits: stats.crits,
				healing: stats.healing,
				hps: (stats.healing as f64 / seconds) as u64,
				taken: stats.taken,
				dtps: (stats.taken as f64 / seconds) as u64,
				taken_hits: stats.taken_hits,
				own: self.own == Some(*id),
				member: self.own.is_none_or(|own| own == *id || encounter.members.contains(id)),
				skills: skills(&stats.skills),
				heals: skills(&stats.heals),
				sources: skills(&stats.sources),
			})
			.collect();
		players.sort_by_key(|player| Reverse(player.damage));
		players
	}

	fn boss(&self, encounter: &Encounter) -> Option<Boss> {
		let (id, _) = encounter.targets.iter().filter(|(target, _)| !self.is_player(**target) && !self.owners.contains_key(target)).max_by_key(|(_, damage)| **damage)?;
		let vitals = self.vitals.get(id).filter(|vitals| vitals.first.is_some())?;
		Some(Boss { npc: vitals.npc, hp: vitals.hp, max: vitals.limit(), estimated: vitals.max.is_none(), dead: vitals.dead })
	}

	fn vote(&mut self, actor: u64, target: u64, skill: u32) {
		if !CLASS_SKILLS.contains(&skill) || self.spawned.contains(&actor) {
			return;
		}
		let votes = self.classes.entry(actor).or_default();
		if actor != target {
			votes[(skill / 1_000_000 - 10) as usize] += 1;
		}
	}

	fn owner(&self, entity: u64) -> u64 {
		self.owners.get(&entity).copied().unwrap_or(entity)
	}

	fn member(&self, entity: u64) -> bool {
		self.own.is_none_or(|own| own == entity || self.party.contains(&entity))
	}

	fn is_player(&self, entity: u64) -> bool {
		!self.spawned.contains(&entity) && (self.classes.contains_key(&entity) || self.names.contains_key(&entity))
	}

	fn class_of(&self, entity: u64) -> u8 {
		self.classes.get(&entity).and_then(|votes| (1..CLASS_COUNT).filter(|class| votes[*class] > 0).max_by_key(|class| votes[*class])).unwrap_or_default() as u8
	}
}

impl Vitals {
	fn limit(&self) -> u64 {
		self.max.unwrap_or_else(|| self.highest.max(self.first.map_or(0, |first| first + self.before)))
	}
}

fn base_skill(skill: u32) -> u32 {
	if CLASS_SKILLS.contains(&skill) && !SPIRIT_SKILLS.iter().any(|spirits| spirits.contains(&skill)) {
		family(skill)
	} else {
		skill
	}
}

fn skills(totals: &HashMap<u32, Totals>) -> Vec<Skill> {
	let mut skills: Vec<Skill> = totals.iter().map(|(id, totals)| Skill { id: *id, amount: totals.amount, hits: totals.hits, crits: totals.crits }).collect();
	skills.sort_by_key(|skill| Reverse(skill.amount));
	skills
}

#[cfg(test)]
mod tests {
	use super::*;

	fn hit(target: u64, actor: u64, skill: u32, damage: u64) -> Event {
		Event::Hit(Hit { target, actor, skill, damage, critical: false, dot: false, drain: 0 })
	}

	fn boss(entity: u64, hp: u64) -> Event {
		Event::Spawn { entity, owner: None, vitals: Some(SpawnVitals { npc: 2301014, hp, max: 5_000_000 }) }
	}

	#[test]
	fn aggregates_player_damage_and_summons() {
		let mut meter = Meter::default();
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, Event::Spawn { entity: 50, owner: Some(2), vitals: None });
		meter.apply(0, Event::Spawn { entity: 51, owner: None, vitals: None });
		meter.apply(1_000_000, hit(900, 1, 14010353, 1000));
		meter.apply(1_000_000, hit(900, 2, 16000010, 500));
		meter.apply(2_000_000, hit(900, 50, 16250001, 250));
		meter.apply(2_000_000, hit(900, 51, 16250001, 300));
		meter.apply(2_000_000, hit(50, 2, 16770001, 999));
		meter.apply(3_000_000, hit(900, 77, 1230790, 9999));
		let snapshot = meter.snapshot(Status::Live);
		assert_eq!(snapshot.total, 1750);
		assert_eq!(snapshot.players.len(), 2);
		assert_eq!(snapshot.players[0].name.as_deref(), Some("Moi"));
		assert_eq!(snapshot.players[0].class, 4);
		assert_eq!(snapshot.players[1].damage, 750);
		assert_eq!(snapshot.players[0].skills[0].id, 14010000);
	}

	#[test]
	fn keeps_majority_class() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 16000010, 10));
		meter.apply(0, hit(900, 1, 16000020, 10));
		meter.apply(0, hit(900, 1, 18160032, 10));
		assert_eq!(meter.snapshot(Status::Live).players[0].class, 6);
	}

	#[test]
	fn keeps_boss_encounter_through_player_revive() {
		let mut meter = Meter::default();
		meter.apply(0, Event::MapChange { map: 600082, revive: false });
		meter.apply(0, boss(900, 5_000_000));
		meter.apply(1_000_000, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::Combat { entity: 900, active: true });
		meter.apply(5_000_000, Event::MapChange { map: 600082, revive: true });
		meter.apply(IDLE_MICROS * 4, hit(900, 1, 11020000, 40));
		assert_eq!(meter.snapshot(Status::Live).total, 140);
	}

	#[test]
	fn ends_encounter_when_boss_dies() {
		let mut meter = Meter::default();
		meter.apply(0, boss(900, 5_000_000));
		meter.apply(1_000_000, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::Combat { entity: 900, active: true });
		meter.apply(2_000_000, Event::Health { entity: 900, hp: 0 });
		meter.apply(2_500_000, hit(900, 1, 11020000, 7));
		meter.apply(3_000_000, hit(901, 1, 11020000, 40));
		let snapshot = meter.snapshot(Status::Live);
		assert_eq!(snapshot.total, 40);
		assert!(snapshot.boss.is_none());
	}

	#[test]
	fn reports_boss_health() {
		let mut meter = Meter::default();
		meter.apply(0, boss(900, 5_000_000));
		meter.apply(1_000_000, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::Health { entity: 900, hp: 4_000_000 });
		let boss = meter.snapshot(Status::Live).boss.unwrap();
		assert_eq!((boss.npc, boss.hp, boss.max, boss.estimated), (Some(2301014), 4_000_000, 5_000_000, false));
	}

	#[test]
	fn counts_heals_and_drain() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, Event::Heal(Heal { target: 1, actor: 2, skill: 17100450, amount: 300 }));
		meter.apply(0, Event::Hit(Hit { target: 900, actor: 1, skill: 13010000, damage: 50, critical: false, dot: false, drain: 20 }));
		let snapshot = meter.snapshot(Status::Live);
		assert_eq!(snapshot.total_healing, 320);
		assert_eq!(snapshot.players.iter().find(|player| player.id == 2).unwrap().heals[0].id, 17100000);
		assert_eq!(snapshot.players.iter().find(|player| player.id == 1).unwrap().heals[0].id, DRAIN_SKILL);
	}

	#[test]
	fn counts_damage_taken_by_source() {
		let mut meter = Meter::default();
		meter.apply(0, boss(900, 5_000_000));
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, hit(1, 900, 1801966, 400));
		meter.apply(1_000_000, Event::Hit(Hit { target: 1, actor: 900, skill: 1801966, damage: 50, critical: false, dot: true, drain: 0 }));
		meter.apply(1_000_000, hit(1, 2, 11020000, 999));
		meter.apply(1_000_000, Event::Hit(Hit { target: 1, actor: 900, skill: 18730002, damage: 2134, critical: false, dot: true, drain: 0 }));
		let snapshot = meter.snapshot(Status::Live);
		let player = snapshot.players.iter().find(|player| player.id == 1).unwrap();
		assert_eq!((snapshot.total_taken, player.taken, player.taken_hits), (450, 450, 1));
		assert_eq!(player.sources[0].id, 2301014);
	}

	#[test]
	fn shows_only_self_and_party() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, Event::PartyMember { entity: 2 });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 2, 12020000, 50));
		meter.apply(0, hit(900, 3, 13020000, 999));
		assert_eq!(meter.snapshot(Status::Live).total, 150);
		meter.set_party_only(false);
		assert_eq!(meter.snapshot(Status::Live).total, 1149);
	}

	#[test]
	fn ignores_strangers_for_encounter_timing() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(5_000_000, hit(900, 3, 13020000, 999));
		meter.apply(IDLE_MICROS * 2, hit(901, 3, 13020000, 999));
		let snapshot = meter.snapshot(Status::Live);
		assert_eq!((snapshot.duration, snapshot.total), (0, 100));
	}

	#[test]
	fn keeps_members_after_party_disbands() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, Event::PartyRoster { entities: HashSet::from([1, 2]) });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 2, 12020000, 50));
		meter.apply(1_000_000, Event::PartyRoster { entities: HashSet::from([1]) });
		assert_eq!(meter.snapshot(Status::Live).total, 150);
	}

	#[test]
	fn removes_members_leaving_the_roster() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, Event::PartyMember { entity: 4 });
		meter.apply(0, Event::PartyRoster { entities: HashSet::from([1, 2, 3]) });
		meter.apply(0, Event::PartyRoster { entities: HashSet::from([1, 3]) });
		for actor in 1..=5 {
			meter.apply(0, hit(900, actor, 11020000, 10));
		}
		let players: HashSet<u64> = meter.snapshot(Status::Live).players.iter().map(|player| player.id).collect();
		assert_eq!(players, HashSet::from([1, 3, 4]));
	}

	#[test]
	fn shows_everyone_until_self_is_known() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, Event::PartyMember { entity: 2 });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 3, 11020000, 100));
		assert_eq!(meter.snapshot(Status::Live).players.len(), 2);
	}

	#[test]
	fn starts_new_encounter_after_idle() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(IDLE_MICROS + 1, hit(901, 1, 11020000, 40));
		assert_eq!(meter.snapshot(Status::Live).total, 40);
	}
}