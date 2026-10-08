use std::cmp::Reverse;
use std::collections::{HashMap, HashSet, VecDeque};
use std::mem;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::packet::{family, Event, Heal, Hit, Vitals as SpawnVitals};

const IDLE_MICROS: u64 = 15_000_000;
const GRACE_MICROS: u64 = 2_000_000;
const BOSS_HP: u64 = 1_000_000;
const CLASS_SKILLS: Range<u32> = 11_000_000..20_000_000;
const SPIRIT_SKILLS: [Range<u32>; 2] = [16_000_000..16_010_000, 16_990_000..17_000_000];
const MONSTER_SKILLS: Range<u32> = 1_000_000..2_000_000;
const CLASS_COUNT: usize = 10;
const DRAIN_SKILL: u32 = 1;
const SECOND: u64 = 1_000_000;
const ACTIVE_GAP: u64 = 3_000_000;
const RECAP: usize = 5;

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
	pub health: Vec<Option<f32>>,
	pub players: Vec<Player>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Boss {
	pub npc: Option<u32>,
	pub hp: u64,
	pub max: u64,
	pub estimated: bool,
	pub dead: bool,
}

#[derive(Serialize, Deserialize, Clone)]
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
	#[serde(default)]
	pub deaths: u64,
	#[serde(default)]
	pub revived: u64,
	#[serde(default)]
	pub resurrections: u64,
	#[serde(default)]
	pub gear: Option<u32>,
	#[serde(default)]
	pub power: Option<u64>,
	#[serde(default)]
	pub perfect: u64,
	#[serde(default)]
	pub hard: u64,
	#[serde(default)]
	pub back: u64,
	#[serde(default)]
	pub front: u64,
	#[serde(default)]
	pub additional: u64,
	#[serde(default)]
	pub active: u64,
	#[serde(default)]
	pub timeline: Vec<u64>,
	#[serde(default)]
	pub evasions: u64,
	#[serde(default)]
	pub resists: u64,
	#[serde(default)]
	pub blocks: u64,
	#[serde(default)]
	pub parries: u64,
	#[serde(default)]
	pub perfect_blocks: u64,
	#[serde(default)]
	pub iron_walls: u64,
	#[serde(default)]
	pub effective: u64,
	#[serde(default)]
	pub recaps: Vec<Recap>,
	#[serde(default)]
	pub buffs: Vec<Uptime>,
	#[serde(default)]
	pub debuffs: Vec<Uptime>,
	pub own: bool,
	pub member: bool,
	pub skills: Vec<Skill>,
	pub heals: Vec<Skill>,
	pub sources: Vec<Skill>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
	pub id: u32,
	pub amount: u64,
	pub hits: u64,
	pub crits: u64,
	#[serde(default)]
	pub perfect: u64,
	#[serde(default)]
	pub hard: u64,
	#[serde(default)]
	pub back: u64,
	#[serde(default)]
	pub casts: u64,
	#[serde(default)]
	pub max: u64,
	#[serde(default)]
	pub effective: u64,
	#[serde(default)]
	pub variants: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recap {
	pub at: u64,
	pub blows: Vec<Blow>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Blow {
	pub at: u64,
	pub npc: u32,
	pub skill: u32,
	pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Uptime {
	pub id: u32,
	pub active: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fight {
	pub start: u64,
	pub last: u64,
	pub boss: Option<Boss>,
	#[serde(default)]
	pub health: Vec<Option<f32>>,
	pub players: Vec<Player>,
}

#[derive(Default)]
pub struct Meter {
	names: HashMap<u64, String>,
	classes: HashMap<u64, [u32; CLASS_COUNT]>,
	spawned: HashSet<u64>,
	owners: HashMap<u64, u64>,
	fallen: HashMap<u64, Option<u64>>,
	buffs: HashMap<(u64, u32, u64), (u64, u64)>,
	vitals: HashMap<u64, Vitals>,
	own: Option<u64>,
	party: HashSet<u64>,
	roster: HashSet<u64>,
	profiles: HashMap<String, (u32, u64)>,
	party_only: bool,
	dungeon: bool,
	map: Option<u32>,
	zone: u32,
	encounter: Option<Encounter>,
	history: Vec<Fight>,
	closed: Vec<Vec<Fight>>,
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
	zone: u32,
	start: u64,
	last: u64,
	ended: Option<u64>,
	targets: HashMap<u64, u64>,
	actors: HashMap<u64, Stats>,
	members: HashSet<u64>,
	uptimes: HashMap<(u64, u32, u64), u64>,
	health: HashMap<u64, Vec<(u64, u64)>>,
	frozen: Option<Fight>,
}

#[derive(Default)]
struct Stats {
	damage: u64,
	hits: u64,
	crits: u64,
	healing: u64,
	taken: u64,
	taken_hits: u64,
	deaths: u64,
	revived: u64,
	resurrections: u64,
	perfect: u64,
	hard: u64,
	back: u64,
	front: u64,
	additional: u64,
	active: u64,
	last: Option<u64>,
	timeline: Vec<u64>,
	evasions: u64,
	resists: u64,
	blocks: u64,
	parries: u64,
	perfect_blocks: u64,
	iron_walls: u64,
	effective: u64,
	recent: VecDeque<Blow>,
	recaps: Vec<Recap>,
	skills: HashMap<u32, Totals>,
	heals: HashMap<u32, Totals>,
	sources: HashMap<u32, Totals>,
}

#[derive(Default)]
struct Totals {
	amount: u64,
	hits: u64,
	crits: u64,
	perfect: u64,
	hard: u64,
	back: u64,
	casts: u64,
	max: u64,
	effective: u64,
	cast: Option<u8>,
	variants: HashSet<u32>,
}

impl Meter {
	pub fn apply(&mut self, micros: u64, event: Event) {
		match event {
			Event::Hit(hit) => self.hit(micros, hit),
			Event::Heal(heal) => self.heal(micros, heal),
			Event::Avoid { target, actor, resisted } => self.avoid(micros, target, actor, resisted),
			Event::Buff { target, caster, effect, duration } => self.buff(micros, target, caster, effect, duration),
			Event::Resurrection { target, actor } => {
				let actor = self.owner(actor);
				if self.is_player(actor) {
					if let Some(resurrector) = self.fallen.get_mut(&target) {
						resurrector.get_or_insert(actor);
					}
				}
			}
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
			Event::MapChange { map, revive, entry } => {
				if self.map != Some(map) || entry || (!revive && !self.boss_in_combat(None)) {
					self.end(micros);
				}
				if self.map != Some(map) || entry {
					self.zone += 1;
				}
				self.map = Some(map);
			}
			Event::PartyMember { entity } => self.join(entity),
			Event::PartyStatus { entity, hp, max } => {
				self.join(entity);
				self.vitals.entry(entity).or_default().max = Some(max);
				self.health(micros, entity, hp);
			}
			Event::PartyRoster { members } => {
				let entities: HashSet<u64> = members.keys().copied().collect();
				for left in self.roster.difference(&entities) {
					self.party.remove(left);
				}
				for (entity, name) in members {
					self.join(entity);
					if let Some(name) = name {
						self.names.insert(entity, name);
					}
				}
				self.roster = entities;
			}
			Event::PartyProfiles { profiles } => {
				for profile in profiles {
					self.profiles.insert(profile.name, (profile.gear, profile.power));
				}
			}
		}
	}

	pub fn reconnect(&mut self, micros: u64) {
		self.end(micros);
		let frozen = self.encounter.as_ref().map(|encounter| self.fight(encounter));
		if let (Some(encounter), Some(frozen)) = (&mut self.encounter, frozen) {
			encounter.frozen.get_or_insert(frozen);
		}
		self.names.clear();
		self.classes.clear();
		self.spawned.clear();
		self.owners.clear();
		self.fallen.clear();
		self.buffs.clear();
		self.vitals.clear();
		self.own = None;
		self.party.clear();
		self.roster.clear();
	}

	pub fn reset(&mut self) {
		if let Some(encounter) = self.encounter.take() {
			self.history.push(self.fight(&encounter));
		}
		self.close();
	}

	pub fn closed(&mut self) -> Vec<Vec<Fight>> {
		mem::take(&mut self.closed)
	}

	pub fn session(&self) -> Vec<Fight> {
		let mut fights = self.history.clone();
		fights.extend(self.encounter.as_ref().map(|encounter| self.fight(encounter)));
		fights
	}

	pub fn set_party_only(&mut self, enabled: bool) {
		self.party_only = enabled;
	}

	pub fn set_dungeon(&mut self, enabled: bool) {
		self.dungeon = enabled;
	}

	pub fn snapshot(&self, status: Status) -> Snapshot {
		let current = self.encounter.as_ref().map(|encounter| self.fight(encounter));
		let mut fights: Vec<&Fight> = Vec::new();
		if self.dungeon {
			fights.extend(&self.history);
		}
		fights.extend(&current);
		summarize(status, &fights, self.party_only)
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
				self.archive();
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
		let drained = self.vitals.get(&actor).map_or(hit.drain, |vitals| hit.drain.min(vitals.limit().saturating_sub(vitals.hp)));
		let encounter = self.encounter.get_or_insert_with(|| Encounter {
			zone: self.zone,
			start: micros,
			last: micros,
			ended: None,
			targets: HashMap::new(),
			actors: HashMap::new(),
			members: self.party.clone(),
			uptimes: HashMap::new(),
			health: HashMap::new(),
			frozen: None,
		});
		if starts {
			encounter.last = encounter.last.max(micros);
			*encounter.targets.entry(hit.target).or_default() += hit.damage;
		}
		let second = (micros.saturating_sub(encounter.start) / SECOND) as usize;
		let stats = encounter.actors.entry(actor).or_default();
		let skill = stats.skills.entry(base_skill(hit.skill)).or_default();
		stats.damage += hit.damage;
		skill.amount += hit.damage;
		skill.variants.insert(hit.skill);
		if stats.timeline.len() <= second {
			stats.timeline.resize(second + 1, 0);
		}
		stats.timeline[second] += hit.damage;
		if !hit.dot {
			let strike = hit.strike;
			stats.hits += 1;
			stats.crits += hit.critical as u64;
			stats.perfect += strike.perfect as u64;
			stats.hard += strike.hard as u64;
			stats.back += strike.back as u64;
			stats.front += strike.front as u64;
			stats.additional += strike.additional as u64;
			skill.hits += 1;
			skill.crits += hit.critical as u64;
			skill.perfect += strike.perfect as u64;
			skill.hard += strike.hard as u64;
			skill.back += strike.back as u64;
			skill.max = skill.max.max(hit.damage);
			if skill.cast != Some(hit.cast) {
				skill.cast = Some(hit.cast);
				skill.casts += 1;
			}
			if let Some(last) = stats.last.filter(|last| micros.saturating_sub(*last) <= ACTIVE_GAP) {
				stats.active += micros.saturating_sub(last);
			}
			stats.last = Some(stats.last.map_or(micros, |last| last.max(micros)));
		}
		if hit.drain > 0 && starts {
			stats.healing += hit.drain;
			stats.effective += drained;
			let drain = stats.heals.entry(DRAIN_SKILL).or_default();
			drain.amount += hit.drain;
			drain.effective += drained;
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
		let effective = self.vitals.get(&heal.target).map_or(heal.amount, |vitals| heal.amount.min(vitals.limit().saturating_sub(vitals.hp)));
		let Some(encounter) = &mut self.encounter else {
			return;
		};
		let stats = encounter.actors.entry(healer).or_default();
		let skill = stats.heals.entry(base_skill(heal.skill)).or_default();
		stats.healing += heal.amount;
		stats.effective += effective;
		skill.amount += heal.amount;
		skill.effective += effective;
		skill.hits += 1;
	}

	fn avoid(&mut self, micros: u64, target: u64, actor: u64, resisted: bool) {
		if !self.is_player(target) || self.is_player(self.owner(actor)) || !self.active(micros) {
			return;
		}
		if let Some(encounter) = &mut self.encounter {
			let stats = encounter.actors.entry(target).or_default();
			if resisted {
				stats.resists += 1;
			} else {
				stats.evasions += 1;
			}
		}
	}

	fn buff(&mut self, micros: u64, target: u64, caster: u64, effect: u32, duration: u32) {
		let family = family(effect);
		if !CLASS_SKILLS.contains(&family) {
			return;
		}
		let key = (target, family, self.owner(caster));
		let end = micros + duration as u64 * 1000;
		let span = self.buffs.entry(key).or_insert((micros, end));
		if micros <= span.1 {
			span.1 = span.1.max(end);
			return;
		}
		let closed = mem::replace(span, (micros, end));
		if let Some(encounter) = &mut self.encounter {
			*encounter.uptimes.entry(key).or_default() += closed.1.saturating_sub(closed.0.max(encounter.start));
		}
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
			let strike = hit.strike;
			stats.taken_hits += 1;
			stats.blocks += strike.blocked as u64;
			stats.parries += strike.parried as u64;
			stats.perfect_blocks += strike.perfect_block as u64;
			stats.iron_walls += strike.iron_wall as u64;
			source.hits += 1;
		}
		stats.recent.push_back(Blow { at: micros, npc, skill: hit.skill, amount: hit.damage });
		if stats.recent.len() > RECAP {
			stats.recent.pop_front();
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
		let player = self.is_player(entity);
		let vitals = self.vitals.entry(entity).or_default();
		let died = player && hp == 0 && !vitals.dead;
		let revived = player && hp > 0 && vitals.dead;
		vitals.hp = hp;
		vitals.highest = vitals.highest.max(hp);
		vitals.first.get_or_insert(hp);
		vitals.dead = hp == 0;
		if hp == 0 {
			vitals.in_combat = false;
		}
		let reset = vitals.max == Some(hp) && vitals.combat_seen && !vitals.in_combat;
		if let Some(encounter) = &mut self.encounter {
			if encounter.targets.contains_key(&entity) {
				encounter.health.entry(entity).or_default().push((micros, hp));
			}
		}
		if hp == 0 || reset {
			self.finish(micros, entity);
		}
		if died {
			self.die(micros, entity);
		}
		if revived {
			self.revive(entity);
		}
	}

	fn die(&mut self, micros: u64, entity: u64) {
		self.fallen.insert(entity, None);
		if !self.active(micros) {
			return;
		}
		if let Some(encounter) = &mut self.encounter {
			let start = encounter.start;
			let stats = encounter.actors.entry(entity).or_default();
			stats.deaths += 1;
			let blows = stats.recent.drain(..).map(|blow| Blow { at: blow.at.saturating_sub(start) / 1000, ..blow }).collect();
			stats.recaps.push(Recap { at: micros.saturating_sub(start) / 1000, blows });
		}
	}

	fn revive(&mut self, entity: u64) {
		let Some(resurrector) = self.fallen.remove(&entity).flatten() else {
			return;
		};
		if let Some(encounter) = &mut self.encounter {
			encounter.actors.entry(entity).or_default().revived += 1;
			encounter.actors.entry(resurrector).or_default().resurrections += 1;
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

	fn archive(&mut self) {
		let Some(encounter) = self.encounter.take() else {
			return;
		};
		self.history.push(self.fight(&encounter));
		if encounter.zone != self.zone {
			self.close();
		}
	}

	fn close(&mut self) {
		if !self.history.is_empty() {
			self.closed.push(mem::take(&mut self.history));
		}
	}

	fn fight(&self, encounter: &Encounter) -> Fight {
		match &encounter.frozen {
			Some(frozen) => frozen.clone(),
			None => {
				let target = self.target(encounter);
				Fight { start: encounter.start, last: encounter.last, boss: self.boss(target), health: self.series(encounter, target), players: self.players(encounter, target) }
			}
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

	fn players(&self, encounter: &Encounter, target: Option<u64>) -> Vec<Player> {
		let elapsed = encounter.last - encounter.start;
		let mut players: Vec<Player> = encounter
			.actors
			.iter()
			.filter(|(id, stats)| self.is_player(**id) && (stats.damage > 0 || stats.healing > 0 || stats.taken > 0 || stats.deaths > 0 || stats.resurrections > 0 || stats.evasions + stats.resists > 0))
			.map(|(id, stats)| (id, stats, self.names.get(id).and_then(|name| self.profiles.get(name))))
			.map(|(id, stats, profile)| Player {
				id: *id,
				name: self.names.get(id).cloned(),
				class: self.class_of(*id),
				damage: stats.damage,
				dps: per_second(stats.damage, elapsed),
				hits: stats.hits,
				crits: stats.crits,
				healing: stats.healing,
				hps: per_second(stats.healing, elapsed),
				taken: stats.taken,
				dtps: per_second(stats.taken, elapsed),
				taken_hits: stats.taken_hits,
				deaths: stats.deaths,
				revived: stats.revived,
				resurrections: stats.resurrections,
				gear: profile.map(|(gear, _)| *gear),
				power: profile.map(|(_, power)| *power),
				perfect: stats.perfect,
				hard: stats.hard,
				back: stats.back,
				front: stats.front,
				additional: stats.additional,
				active: stats.active / 1000,
				timeline: stats.timeline.clone(),
				evasions: stats.evasions,
				resists: stats.resists,
				blocks: stats.blocks,
				parries: stats.parries,
				perfect_blocks: stats.perfect_blocks,
				iron_walls: stats.iron_walls,
				effective: stats.effective,
				recaps: stats.recaps.clone(),
				buffs: self.uptimes(encounter, |(buffed, _, _)| buffed == id),
				debuffs: self.uptimes(encounter, |(buffed, _, caster)| caster == id && Some(*buffed) == target),
				own: self.own == Some(*id),
				member: self.own == Some(*id) || encounter.members.contains(id),
				skills: skills(&stats.skills),
				heals: skills(&stats.heals),
				sources: skills(&stats.sources),
			})
			.collect();
		players.sort_by_key(|player| Reverse(player.damage));
		players
	}

	fn target(&self, encounter: &Encounter) -> Option<u64> {
		encounter.targets.iter().filter(|(target, _)| !self.is_player(**target) && !self.owners.contains_key(target)).max_by_key(|(_, damage)| **damage).map(|(target, _)| *target)
	}

	fn boss(&self, target: Option<u64>) -> Option<Boss> {
		let vitals = self.vitals.get(&target?).filter(|vitals| vitals.first.is_some())?;
		Some(Boss { npc: vitals.npc, hp: vitals.hp, max: vitals.limit(), estimated: vitals.max.is_none(), dead: vitals.dead })
	}

	fn series(&self, encounter: &Encounter, target: Option<u64>) -> Vec<Option<f32>> {
		let Some(target) = target else {
			return Vec::new();
		};
		let limit = self.vitals.get(&target).map_or(0, Vitals::limit).max(1) as f32;
		let mut samples = encounter.health.get(&target).into_iter().flatten().peekable();
		let mut current = None;
		(0..=(encounter.last - encounter.start) / SECOND)
			.map(|second| {
				let until = encounter.start + (second + 1) * SECOND;
				while let Some((_, hp)) = samples.next_if(|(at, _)| *at < until) {
					current = Some((*hp as f32 * 100.0 / limit).min(100.0));
				}
				current
			})
			.collect()
	}

	fn uptimes(&self, encounter: &Encounter, matches: impl Fn(&(u64, u32, u64)) -> bool) -> Vec<Uptime> {
		let elapsed = encounter.last - encounter.start;
		let mut families: HashMap<u32, u64> = HashMap::new();
		for (key, span) in self.buffs.iter().filter(|(key, _)| matches(key)) {
			let open = span.1.min(encounter.last).saturating_sub(span.0.max(encounter.start));
			let active = (encounter.uptimes.get(key).copied().unwrap_or_default() + open).min(elapsed) / 1000;
			let family = families.entry(key.1).or_default();
			*family = (*family).max(active);
		}
		let mut uptimes: Vec<Uptime> = families.into_iter().filter(|(_, active)| *active > 0).map(|(id, active)| Uptime { id, active }).collect();
		uptimes.sort_by_key(|uptime| Reverse(uptime.active));
		uptimes
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
		self.own == Some(entity) || self.party.contains(&entity)
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

pub fn summarize(status: Status, fights: &[&Fight], party_only: bool) -> Snapshot {
	let (Some(first), Some(last)) = (fights.first(), fights.last()) else {
		return Snapshot { status, duration: 0, total: 0, total_healing: 0, total_taken: 0, boss: None, health: Vec::new(), players: Vec::new() };
	};
	let elapsed = last.last - first.start;
	let mut players = Vec::new();
	let mut health = vec![None; (elapsed / SECOND) as usize + 1];
	for fight in fights.iter().rev() {
		let offset = fight.start - first.start;
		for player in &fight.players {
			merge(&mut players, player, offset);
		}
		let start = (offset / SECOND) as usize;
		for (slot, percent) in health.iter_mut().skip(start).zip(&fight.health) {
			*slot = slot.or(*percent);
		}
	}
	for player in &mut players {
		player.dps = per_second(player.damage, elapsed);
		player.hps = per_second(player.healing, elapsed);
		player.dtps = per_second(player.taken, elapsed);
		player.recaps.sort_by_key(|recap| recap.at);
	}
	players.sort_by_key(|player| Reverse(player.damage));
	if party_only {
		players.retain(|player| player.member);
	}
	Snapshot {
		status,
		duration: elapsed / 1000,
		total: players.iter().map(|player| player.damage).sum(),
		total_healing: players.iter().map(|player| player.healing).sum(),
		total_taken: players.iter().map(|player| player.taken).sum(),
		boss: last.boss.clone(),
		health,
		players,
	}
}

fn per_second(amount: u64, micros: u64) -> u64 {
	(amount as f64 / (micros as f64 / 1_000_000.0).max(1.0)) as u64
}

fn merge(players: &mut Vec<Player>, fighter: &Player, offset: u64) {
	let Some(player) = players.iter_mut().find(|player| player.id == fighter.id) else {
		let mut player = Player { timeline: Vec::new(), recaps: Vec::new(), ..fighter.clone() };
		spread(&mut player.timeline, &fighter.timeline, offset);
		player.recaps.extend(shifted(&fighter.recaps, offset));
		players.push(player);
		return;
	};
	player.name = player.name.take().or_else(|| fighter.name.clone());
	if player.class == 0 {
		player.class = fighter.class;
	}
	player.damage += fighter.damage;
	player.hits += fighter.hits;
	player.crits += fighter.crits;
	player.healing += fighter.healing;
	player.taken += fighter.taken;
	player.taken_hits += fighter.taken_hits;
	player.deaths += fighter.deaths;
	player.revived += fighter.revived;
	player.resurrections += fighter.resurrections;
	player.gear = player.gear.or(fighter.gear);
	player.power = player.power.or(fighter.power);
	player.perfect += fighter.perfect;
	player.hard += fighter.hard;
	player.back += fighter.back;
	player.front += fighter.front;
	player.additional += fighter.additional;
	player.active += fighter.active;
	player.evasions += fighter.evasions;
	player.resists += fighter.resists;
	player.blocks += fighter.blocks;
	player.parries += fighter.parries;
	player.perfect_blocks += fighter.perfect_blocks;
	player.iron_walls += fighter.iron_walls;
	player.effective += fighter.effective;
	player.own |= fighter.own;
	player.member |= fighter.member;
	spread(&mut player.timeline, &fighter.timeline, offset);
	player.recaps.extend(shifted(&fighter.recaps, offset));
	accumulate(&mut player.buffs, &fighter.buffs);
	accumulate(&mut player.debuffs, &fighter.debuffs);
	combine(&mut player.skills, &fighter.skills);
	combine(&mut player.heals, &fighter.heals);
	combine(&mut player.sources, &fighter.sources);
}

fn spread(timeline: &mut Vec<u64>, other: &[u64], offset: u64) {
	let start = (offset / SECOND) as usize;
	if timeline.len() < start + other.len() {
		timeline.resize(start + other.len(), 0);
	}
	for (slot, value) in timeline.iter_mut().skip(start).zip(other) {
		*slot += value;
	}
}

fn shifted(recaps: &[Recap], offset: u64) -> impl Iterator<Item = Recap> + '_ {
	let delay = offset / 1000;
	recaps.iter().map(move |recap| Recap { at: recap.at + delay, blows: recap.blows.iter().map(|blow| Blow { at: blow.at + delay, ..*blow }).collect() })
}

fn accumulate(uptimes: &mut Vec<Uptime>, others: &[Uptime]) {
	for other in others {
		match uptimes.iter_mut().find(|uptime| uptime.id == other.id) {
			Some(uptime) => uptime.active += other.active,
			None => uptimes.push(other.clone()),
		}
	}
	uptimes.sort_by_key(|uptime| Reverse(uptime.active));
}

fn combine(skills: &mut Vec<Skill>, others: &[Skill]) {
	for other in others {
		match skills.iter_mut().find(|skill| skill.id == other.id) {
			Some(skill) => {
				skill.amount += other.amount;
				skill.hits += other.hits;
				skill.crits += other.crits;
				skill.perfect += other.perfect;
				skill.hard += other.hard;
				skill.back += other.back;
				skill.casts += other.casts;
				skill.max = skill.max.max(other.max);
				skill.effective += other.effective;
				for variant in &other.variants {
					if !skill.variants.contains(variant) {
						skill.variants.push(*variant);
					}
				}
				skill.variants.sort_unstable();
			}
			None => skills.push(other.clone()),
		}
	}
	skills.sort_by_key(|skill| Reverse(skill.amount));
}

fn skills(totals: &HashMap<u32, Totals>) -> Vec<Skill> {
	let mut skills: Vec<Skill> = totals
		.iter()
		.map(|(id, totals)| {
			let mut variants: Vec<u32> = totals.variants.iter().copied().collect();
			variants.sort_unstable();
			Skill {
				id: *id,
				amount: totals.amount,
				hits: totals.hits,
				crits: totals.crits,
				perfect: totals.perfect,
				hard: totals.hard,
				back: totals.back,
				casts: totals.casts,
				max: totals.max,
				effective: totals.effective,
				variants,
			}
		})
		.collect();
	skills.sort_by_key(|skill| Reverse(skill.amount));
	skills
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::packet::{Profile, Strike};

	fn hit(target: u64, actor: u64, skill: u32, damage: u64) -> Event {
		Event::Hit(Hit { target, actor, skill, damage, ..Hit::default() })
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
		meter.apply(0, Event::MapChange { map: 600082, revive: false, entry: true });
		meter.apply(0, boss(900, 5_000_000));
		meter.apply(1_000_000, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::Combat { entity: 900, active: true });
		meter.apply(5_000_000, Event::MapChange { map: 600082, revive: true, entry: false });
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
		meter.apply(0, Event::Hit(Hit { target: 900, actor: 1, skill: 13010000, damage: 50, dot: false, drain: 20, ..Hit::default() }));
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
		meter.apply(1_000_000, Event::Hit(Hit { target: 1, actor: 900, skill: 1801966, damage: 50, dot: true, drain: 0, ..Hit::default() }));
		meter.apply(1_000_000, hit(1, 2, 11020000, 999));
		meter.apply(1_000_000, Event::Hit(Hit { target: 1, actor: 900, skill: 18730002, damage: 2134, dot: true, drain: 0, ..Hit::default() }));
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
		meter.apply(0, Event::PartyRoster { members: HashMap::from([(1, None), (2, None)]) });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 2, 12020000, 50));
		meter.apply(1_000_000, Event::PartyRoster { members: HashMap::from([(1, None)]) });
		assert_eq!(meter.snapshot(Status::Live).total, 150);
	}

	#[test]
	fn removes_members_leaving_the_roster() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, Event::Character { entity: 1, name: String::from("Moi"), own: true });
		meter.apply(0, Event::PartyMember { entity: 4 });
		meter.apply(0, Event::PartyRoster { members: HashMap::from([(1, None), (2, None), (3, None)]) });
		meter.apply(0, Event::PartyRoster { members: HashMap::from([(1, None), (3, None)]) });
		for actor in 1..=5 {
			meter.apply(0, hit(900, actor, 11020000, 10));
		}
		let players: HashSet<u64> = meter.snapshot(Status::Live).players.iter().map(|player| player.id).collect();
		assert_eq!(players, HashSet::from([1, 3, 4]));
	}

	#[test]
	fn names_party_members_from_roster() {
		let mut meter = Meter::default();
		meter.apply(0, Event::PartyRoster { members: HashMap::from([(2, Some(String::from("Alpha")))]) });
		meter.apply(0, hit(900, 2, 12020000, 50));
		assert_eq!(meter.snapshot(Status::Live).players[0].name.as_deref(), Some("Alpha"));
	}

	#[test]
	fn attaches_profiles_by_name() {
		let mut meter = Meter::default();
		meter.apply(0, Event::Character { entity: 2, name: String::from("Alpha"), own: false });
		meter.apply(0, Event::PartyProfiles { profiles: vec![Profile { name: String::from("Alpha"), gear: 1494, power: 72079 }] });
		meter.apply(0, hit(900, 2, 12020000, 50));
		let player = &meter.snapshot(Status::Live).players[0];
		assert_eq!((player.gear, player.power), (Some(1494), Some(72079)));
	}

	#[test]
	fn shows_nobody_without_party_info() {
		let mut meter = Meter::default();
		meter.set_party_only(true);
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 3, 11020000, 100));
		assert!(meter.snapshot(Status::Live).players.is_empty());
		meter.apply(1_000_000, Event::PartyMember { entity: 1 });
		meter.apply(1_000_000, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, hit(900, 3, 11020000, 100));
		let players: Vec<u64> = meter.snapshot(Status::Live).players.iter().map(|player| player.id).collect();
		assert_eq!(players, vec![1]);
	}

	#[test]
	fn accumulates_whole_dungeon() {
		let mut meter = Meter::default();
		meter.set_dungeon(true);
		meter.apply(0, Event::MapChange { map: 600082, revive: false, entry: true });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::MapChange { map: 600082, revive: false, entry: false });
		meter.apply(IDLE_MICROS * 2, hit(901, 1, 11020000, 40));
		let snapshot = meter.snapshot(Status::Live);
		assert_eq!((snapshot.total, snapshot.duration, snapshot.players[0].dps), (140, IDLE_MICROS * 2 / 1000, 4));
		assert_eq!((snapshot.players[0].skills.len(), snapshot.players[0].skills[0].amount), (1, 140));
		meter.set_dungeon(false);
		assert_eq!(meter.snapshot(Status::Live).total, 40);
	}

	#[test]
	fn restarts_dungeon_total_on_new_map() {
		let mut meter = Meter::default();
		meter.set_dungeon(true);
		meter.apply(0, Event::MapChange { map: 600082, revive: false, entry: true });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::MapChange { map: 100001, revive: false, entry: true });
		assert_eq!(meter.snapshot(Status::Live).total, 100);
		meter.apply(2_000_000, hit(901, 1, 11020000, 40));
		assert_eq!(meter.snapshot(Status::Live).total, 40);
	}

	#[test]
	fn restarts_dungeon_total_on_reentry() {
		let mut meter = Meter::default();
		meter.set_dungeon(true);
		meter.apply(0, Event::MapChange { map: 600082, revive: false, entry: true });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, Event::MapChange { map: 600082, revive: false, entry: true });
		meter.apply(2_000_000, hit(901, 1, 11020000, 40));
		assert_eq!(meter.snapshot(Status::Live).total, 40);
	}

	#[test]
	fn closes_sessions_on_new_zone_and_reset() {
		let mut meter = Meter::default();
		meter.apply(0, Event::MapChange { map: 600082, revive: false, entry: true });
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(IDLE_MICROS * 2, hit(901, 1, 11020000, 40));
		meter.apply(IDLE_MICROS * 3, Event::MapChange { map: 100001, revive: false, entry: true });
		meter.apply(IDLE_MICROS * 4, hit(902, 1, 11020000, 10));
		let closed = meter.closed();
		assert_eq!((closed.len(), closed[0].len(), meter.session().len()), (1, 2, 1));
		meter.reset();
		assert_eq!((meter.closed().len(), meter.session().len()), (1, 0));
	}

	#[test]
	fn counts_deaths_and_cleric_resurrections() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 2, 17010000, 100));
		meter.apply(1_000_000, Event::Health { entity: 1, hp: 0 });
		meter.apply(1_000_000, Event::Health { entity: 1, hp: 0 });
		meter.apply(2_000_000, Event::Resurrection { target: 1, actor: 2 });
		meter.apply(3_000_000, Event::Health { entity: 1, hp: 500 });
		meter.apply(4_000_000, Event::Health { entity: 1, hp: 0 });
		meter.apply(5_000_000, Event::Health { entity: 1, hp: 800 });
		let snapshot = meter.snapshot(Status::Live);
		let player = |id| snapshot.players.iter().find(|player| player.id == id).unwrap();
		assert_eq!((player(1).deaths, player(1).revived, player(2).resurrections), (2, 1, 1));
	}

	#[test]
	fn tracks_hit_quality_and_casts() {
		let mut meter = Meter::default();
		let strike = Strike { perfect: true, back: true, ..Strike::default() };
		meter.apply(0, Event::Hit(Hit { target: 900, actor: 1, skill: 11020010, damage: 100, cast: 1, strike, ..Hit::default() }));
		meter.apply(0, Event::Hit(Hit { target: 901, actor: 1, skill: 11020010, damage: 300, cast: 1, ..Hit::default() }));
		meter.apply(2_000_000, Event::Hit(Hit { target: 900, actor: 1, skill: 11020030, damage: 50, cast: 2, ..Hit::default() }));
		meter.apply(9_000_000, Event::Hit(Hit { target: 900, actor: 1, skill: 11020010, damage: 50, cast: 3, ..Hit::default() }));
		let snapshot = meter.snapshot(Status::Live);
		let player = &snapshot.players[0];
		let skill = &player.skills[0];
		assert_eq!((player.perfect, player.back, player.active), (1, 1, 2000));
		assert_eq!((skill.casts, skill.max, skill.variants.clone()), (3, 300, vec![11020010, 11020030]));
		assert_eq!(player.timeline, vec![400, 0, 50, 0, 0, 0, 0, 0, 0, 50]);
	}

	#[test]
	fn builds_death_recap() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(1_000_000, hit(1, 900, 1801966, 400));
		meter.apply(2_000_000, hit(1, 900, 1801967, 600));
		meter.apply(2_500_000, Event::Health { entity: 1, hp: 0 });
		let snapshot = meter.snapshot(Status::Live);
		let recap = &snapshot.players[0].recaps[0];
		assert_eq!((recap.at, recap.blows.len(), recap.blows[1].amount, recap.blows[1].at), (2500, 2, 600, 2000));
	}

	#[test]
	fn measures_buff_uptime() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, Event::Buff { target: 1, caster: 2, effect: 18160041, duration: 2000 });
		meter.apply(0, Event::Buff { target: 900, caster: 1, effect: 11730007, duration: 20000 });
		meter.apply(1_000_000, Event::Buff { target: 1, caster: 2, effect: 18160041, duration: 2000 });
		meter.apply(8_000_000, Event::Buff { target: 1, caster: 2, effect: 18160041, duration: 2000 });
		meter.apply(10_000_000, hit(900, 1, 11020000, 100));
		let snapshot = meter.snapshot(Status::Live);
		let player = &snapshot.players[0];
		assert_eq!((player.buffs[0].id, player.buffs[0].active), (18160000, 5000));
		assert_eq!((player.debuffs[0].id, player.debuffs[0].active), (11730000, 10000));
	}

	#[test]
	fn estimates_effective_healing() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(0, hit(900, 2, 17010000, 100));
		meter.apply(0, Event::PartyStatus { entity: 1, hp: 900, max: 1000 });
		meter.apply(1_000_000, Event::Heal(Heal { target: 1, actor: 2, skill: 17100450, amount: 300 }));
		let snapshot = meter.snapshot(Status::Live);
		let healer = snapshot.players.iter().find(|player| player.id == 2).unwrap();
		assert_eq!((healer.healing, healer.effective), (300, 100));
	}

	#[test]
	fn starts_new_encounter_after_idle() {
		let mut meter = Meter::default();
		meter.apply(0, hit(900, 1, 11020000, 100));
		meter.apply(IDLE_MICROS + 1, hit(901, 1, 11020000, 40));
		assert_eq!(meter.snapshot(Status::Live).total, 40);
	}
}