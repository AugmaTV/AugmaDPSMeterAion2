use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};

use crate::reader::Reader;

const RECORD: [u8; 2] = [0x04, 0x38];
const PERIODIC: [u8; 2] = [0x05, 0x38];
const OWN_CHARACTER: [u8; 2] = [0x33, 0x36];
const OTHER_CHARACTER: [u8; 2] = [0x45, 0x36];
const SPAWN: [u8; 2] = [0x41, 0x36];
const DESPAWN: [u8; 2] = [0x42, 0x36];
const HEALTH: [u8; 2] = [0x00, 0x8D];
const COMBAT: [u8; 2] = [0x21, 0x8D];
const MAP_CHANGE: [u8; 2] = [0x21, 0x36];
const PARTY_ROSTER: [u8; 2] = [0x00, 0x92];
const PARTY_MEMBERS: [[u8; 2]; 2] = [[0x0E, 0x92], [0x1A, 0x92]];
const PARTY_STATUS: [u8; 2] = [0x1B, 0x92];
const BUFF_APPLY: [u8; 2] = [0x2A, 0x38];
const BUFF_REFRESH: [u8; 2] = [0x2B, 0x38];
const PARTY_PROFILES: [u8; 2] = [0x02, 0x97];
const PARENT_MARKER: [u8; 8] = [0xFF; 8];
const ZONE_FLAG: u8 = 0x01;
const ZONE_POSITION: Range<usize> = 2..14;
const ZONE_MARKER: [u8; 2] = [0x07, 0x02];

const CRITICAL: u64 = 3;
const MISS: u64 = 1;
const RESIST: u64 = 6;
const ADDITIONAL: u64 = 0x20;
const BLOCK: u8 = 0x01;
const PARRY: u8 = 0x02;
const PERFECT: u8 = 0x04;
const HARD: u8 = 0x08;
const IRON_WALL: u8 = 0x10;
const PERFECT_BLOCK: u8 = 0x40;
const BACK: u8 = 1;
const FRONT: u8 = 2;
const BUFF_TAG: u8 = 0x13;
const PERMANENT: u32 = u32::MAX;
const PLAIN_SWITCH: u64 = 0x04;
const DRAIN_FLAG: u64 = 0x04;
const DEAD_REASON: u8 = 3;
const REVIVE_REASON: u8 = 4;
const ENTRY_REASON: u8 = 0;
const HEALTH_STAT: u8 = 0;
const NPC_CODES: RangeInclusive<u32> = 2_000_000..=2_999_999;
const SPAWN_MASKS: [usize; 2] = [4, 2];
const GROUP_HEALS: [u32; 5] = [17_100_000, 17_120_000, 17_290_000, 17_800_000, 18_120_000];
const SELF_HEALS: [u32; 18] = [
	11_260_000, 11_720_000, 11_730_000, 12_260_000, 12_350_000, 12_720_000, 13_260_000, 14_710_000, 16_190_000, 16_770_000, 17_240_000, 17_320_000, 17_720_000, 18_160_000, 18_200_000, 18_420_000, 18_720_000, 19_450_000,
];
const SPIRIT_HEALS: [u32; 2] = [16_190_000, 16_770_000];
const NPC_HEALS: [u32; 1] = [1_801_892];
const RESURRECTION: u32 = 17_390_000;
const HOT_HEALS: [u32; 4] = [12_350_000, 16_190_000, 17_090_000, 18_120_000];
const GROUP_HOTS: [u32; 2] = [17_090_000, 18_120_000];
const KIND_REMAINING: u8 = 0x02;
const KIND_VALUE: u8 = 0x01;
const KIND_SUFFIX: u8 = 0x08;
const KIND_OTHER: u8 = 0x30;
const KIND_EXTRA: u8 = 0x40;
const HOT_TICK: u8 = 0x0B;
const HOT_LAST_TICK: u8 = 0x0A;
const UUID_MARKER: u8 = 0x24;
const UUID_LENGTH: usize = 36;
const UUID_DASHES: [usize; 4] = [8, 13, 18, 23];
const ROSTER_ENTITY_OFFSET: usize = 8;
const ROSTER_NAME_OFFSET: usize = 8;
const PROFILE_FIELDS: u8 = 0x1F;
const PROFILE_CONQUEROR: u8 = 0x01;
const PROFILE_READY: u8 = 0x02;
const PROFILE_ORIGIN: u8 = 0x04;
const PROFILE_SERVER: u8 = 0x08;
const PROFILE_REBIRTH: u8 = 0x10;

#[derive(Debug, PartialEq)]
pub enum Event {
	Hit(Hit),
	Heal(Heal),
	Avoid { target: u64, actor: u64, resisted: bool },
	Buff { target: u64, caster: u64, effect: u32, duration: u32 },
	Resurrection { target: u64, actor: u64 },
	Character { entity: u64, name: String, own: bool },
	Spawn { entity: u64, owner: Option<u64>, vitals: Option<Vitals> },
	Health { entity: u64, hp: u64 },
	Despawn { entity: u64, dead: bool },
	Combat { entity: u64, active: bool },
	MapChange { map: u32, revive: bool, entry: bool },
	PartyMember { entity: u64 },
	PartyStatus { entity: u64, hp: u64, max: u64 },
	PartyRoster { members: HashMap<u64, Option<String>> },
	PartyProfiles { profiles: Vec<Profile> },
}

#[derive(Debug, PartialEq, Default)]
pub struct Hit {
	pub target: u64,
	pub actor: u64,
	pub skill: u32,
	pub damage: u64,
	pub critical: bool,
	pub dot: bool,
	pub drain: u64,
	pub cast: u8,
	pub strike: Strike,
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Strike {
	pub perfect: bool,
	pub hard: bool,
	pub back: bool,
	pub front: bool,
	pub additional: bool,
	pub blocked: bool,
	pub parried: bool,
	pub perfect_block: bool,
	pub iron_wall: bool,
}

#[derive(Debug, PartialEq)]
pub struct Heal {
	pub target: u64,
	pub actor: u64,
	pub skill: u32,
	pub amount: u64,
}

#[derive(Debug, PartialEq)]
pub struct Profile {
	pub name: String,
	pub gear: u32,
	pub power: u64,
}

#[derive(Debug, PartialEq)]
pub struct Vitals {
	pub npc: u32,
	pub hp: u64,
	pub max: u64,
}

pub fn decode(body: &[u8]) -> Option<Event> {
	let mut reader = Reader::new(body.get(2..)?);
	match [body[0], body[1]] {
		RECORD => record(&mut reader),
		PERIODIC => periodic(&mut reader),
		OWN_CHARACTER => character(&mut reader, true),
		OTHER_CHARACTER => character(&mut reader, false),
		SPAWN => spawn(&mut reader),
		DESPAWN => despawn(&mut reader),
		HEALTH => health(&mut reader),
		COMBAT => combat(&mut reader),
		MAP_CHANGE => map_change(&mut reader),
		PARTY_ROSTER => party_roster(reader.rest()),
		PARTY_PROFILES => party_profiles(&mut reader),
		PARTY_STATUS => party_status(&mut reader),
		BUFF_APPLY => buff(&mut reader, true),
		BUFF_REFRESH => buff(&mut reader, false),
		opcode if PARTY_MEMBERS.contains(&opcode) => Some(Event::PartyMember { entity: reader.varint()? }),
		_ => None,
	}
}

pub fn family(skill: u32) -> u32 {
	skill - skill % 10_000
}

fn record(reader: &mut Reader) -> Option<Event> {
	let target = reader.varint()?;
	let switch = reader.varint()?;
	let flag = reader.varint()?;
	let actor = reader.varint()?;
	let skill = reader.u32()?;
	let cast = reader.u8()?;
	let kind = reader.varint()?;
	let layout = switch & 0x0F;
	if family(skill) == RESURRECTION {
		return (actor != target).then_some(Event::Resurrection { target, actor });
	}
	if kind == MISS || kind == RESIST {
		return Some(Event::Avoid { target, actor, resisted: kind == RESIST });
	}
	if layout & 0x04 == 0 || skill == 0 {
		return None;
	}
	let mut strike = Strike { additional: switch & ADDITIONAL != 0, ..Strike::default() };
	if layout & 0x02 != 0 {
		let flags = reader.u8()?;
		reader.varint()?;
		let direction = reader.u8()?;
		strike = Strike {
			perfect: flags & PERFECT != 0,
			hard: flags & HARD != 0,
			back: direction == BACK,
			front: direction == FRONT,
			blocked: flags & BLOCK != 0,
			parried: flags & PARRY != 0,
			perfect_block: flags & PERFECT_BLOCK != 0,
			iron_wall: flags & IRON_WALL != 0,
			..strike
		};
	}
	reader.skip(8)?;
	reader.varint()?;
	let amount = reader.varint()?;
	let family = family(skill);
	if switch == PLAIN_SWITCH && (GROUP_HEALS.contains(&family) || (SELF_HEALS.contains(&family) && actor == target)) {
		return Some(Event::Heal(Heal { target, actor, skill, amount }));
	}
	if (SPIRIT_HEALS.contains(&family) && actor != target) || NPC_HEALS.contains(&skill) {
		return None;
	}
	let drain = if flag & DRAIN_FLAG != 0 { drain(reader, layout, switch).unwrap_or_default() } else { 0 };
	Some(Event::Hit(Hit { target, actor, skill, damage: amount, critical: kind == CRITICAL, dot: false, drain, cast, strike }))
}

fn drain(reader: &mut Reader, layout: u64, switch: u64) -> Option<u64> {
	if layout == 4 && switch & 0x10 != 0 {
		reader.varint()?;
	}
	if switch & 0x20 != 0 {
		for _ in 0..reader.varint()? {
			reader.varint()?;
		}
	}
	if switch & 0x40 != 0 {
		let count = reader.varint()? as usize;
		reader.skip(count.checked_mul(4)?)?;
	}
	reader.skip(1)?;
	if reader.u8()? != 0 {
		return None;
	}
	reader.varint()
}

fn periodic(reader: &mut Reader) -> Option<Event> {
	let target = reader.varint()?;
	let kind = reader.u8()?;
	let actor = reader.varint()?;
	reader.varint()?;
	let skill = reader.u32()? / 100;
	if kind & KIND_OTHER != 0 {
		return None;
	}
	let remaining = if kind & KIND_REMAINING != 0 { Some(reader.varint()?) } else { None };
	let value = if kind & KIND_VALUE != 0 { Some(reader.varint()?) } else { None };
	if kind & KIND_EXTRA != 0 {
		reader.skip(4)?;
	}
	let suffix = if kind & KIND_SUFFIX != 0 { Some(reader.u32()?) } else { None };
	let family = family(skill);
	let consistent = suffix.is_none_or(|suffix| self::family(suffix) == family);
	if HOT_HEALS.contains(&family) {
		if !consistent || actor == 0 || !(GROUP_HOTS.contains(&family) || actor == target) {
			return None;
		}
		let amount = match kind {
			HOT_TICK => value?,
			HOT_LAST_TICK => remaining?,
			_ => return None,
		};
		return Some(Event::Heal(Heal { target, actor, skill, amount }));
	}
	if kind & KIND_VALUE != 0 || actor == target || !consistent {
		return None;
	}
	Some(Event::Hit(Hit { target, actor, skill, damage: remaining?, dot: true, ..Hit::default() }))
}

fn character(reader: &mut Reader, own: bool) -> Option<Event> {
	let entity = reader.varint()?;
	reader.skip(4)?;
	if reader.u8()? & 1 == 0 {
		return None;
	}
	let name = reader.string()?;
	Some(Event::Character { entity, name, own })
}

fn spawn(reader: &mut Reader) -> Option<Event> {
	let entity = reader.varint()?;
	let vitals = SPAWN_MASKS.into_iter().find_map(|width| vitals(reader.rest(), width));
	let owner = reader.seek(&PARENT_MARKER).and_then(|_| {
		reader.skip(8)?;
		match reader.varint()? {
			parent if parent == entity => zone_owner(reader),
			parent => Some(parent),
		}
	});
	Some(Event::Spawn { entity, owner, vitals })
}

fn zone_owner(reader: &Reader) -> Option<u64> {
	let rest = reader.rest();
	if rest.first() != Some(&ZONE_FLAG) {
		return None;
	}
	let marker = [rest.get(ZONE_POSITION)?, &ZONE_MARKER].concat();
	let start = rest.windows(marker.len()).position(|window| window == marker)?;
	Reader::new(rest.get(start + marker.len() + 1..)?).u32().map(u64::from)
}

fn vitals(data: &[u8], width: usize) -> Option<Vitals> {
	let mut reader = Reader::new(data);
	reader.skip(width)?;
	if reader.u8()? & 1 != 0 {
		let length = reader.varint()? as usize;
		reader.skip(length)?;
	}
	let npc = reader.u32()?;
	if !NPC_CODES.contains(&npc) {
		return None;
	}
	let movement = reader.u8()?;
	reader.skip(17)?;
	if movement & 0x08 != 0 {
		reader.skip(12)?;
	}
	reader.skip(3)?;
	let hp = reader.varint()?;
	let max = reader.varint()?;
	(max > 0 && hp <= max).then_some(Vitals { npc, hp, max })
}

fn despawn(reader: &mut Reader) -> Option<Event> {
	let entity = reader.varint()?;
	reader.varint()?;
	Some(Event::Despawn { entity, dead: reader.u8()? == DEAD_REASON })
}

fn health(reader: &mut Reader) -> Option<Event> {
	let entity = reader.varint()?;
	let mask = reader.u8()?;
	if mask & 0x01 != 0 {
		let count = reader.u8()? as usize;
		reader.skip(count * 5)?;
	}
	if mask & 0x02 == 0 {
		return None;
	}
	let mut hp = None;
	for _ in 0..reader.u8()? {
		let stat = reader.u8()?;
		let value = reader.u64()?;
		if stat == HEALTH_STAT {
			hp = Some(value);
		}
	}
	Some(Event::Health { entity, hp: hp? })
}

fn combat(reader: &mut Reader) -> Option<Event> {
	let entity = reader.varint()?;
	reader.skip(1)?;
	Some(Event::Combat { entity, active: reader.u8()? == 1 })
}

fn map_change(reader: &mut Reader) -> Option<Event> {
	reader.skip(4)?;
	let map = reader.u32()?;
	reader.skip(24)?;
	let reason = reader.u8()?;
	Some(Event::MapChange { map, revive: reason == REVIVE_REASON, entry: reason == ENTRY_REASON })
}

fn party_roster(data: &[u8]) -> Option<Event> {
	let members: HashMap<u64, Option<String>> = (ROSTER_ENTITY_OFFSET..data.len())
		.filter(|anchor| data[*anchor] == UUID_MARKER && data.get(anchor + 1..anchor + 1 + UUID_LENGTH).is_some_and(uuid))
		.filter_map(|anchor| {
			let entity = Reader::new(&data[anchor - ROSTER_ENTITY_OFFSET..]).u32()?;
			let mut reader = Reader::new(&data[anchor + 1 + UUID_LENGTH..]);
			let name = reader.skip(ROSTER_NAME_OFFSET).and_then(|_| reader.string());
			Some((u64::from(entity), name))
		})
		.filter(|(entity, _)| *entity != 0)
		.collect();
	(!members.is_empty()).then_some(Event::PartyRoster { members })
}

fn party_status(reader: &mut Reader) -> Option<Event> {
	let entity = reader.varint()?;
	let hp = reader.varint()?;
	let max = reader.varint()?;
	Some(Event::PartyStatus { entity, hp, max })
}

fn buff(reader: &mut Reader, applied: bool) -> Option<Event> {
	let target = reader.varint()?;
	if applied {
		reader.skip(1)?;
	}
	if reader.u8()? != BUFF_TAG {
		return None;
	}
	reader.varint()?;
	let effect = reader.u32()? / 10;
	let duration = reader.u32()?;
	reader.skip(12)?;
	let caster = reader.varint()?;
	(duration != PERMANENT).then_some(Event::Buff { target, caster, effect, duration })
}

fn party_profiles(reader: &mut Reader) -> Option<Event> {
	reader.skip(4)?;
	reader.string()?;
	reader.skip(15)?;
	reader.bit()?;
	reader.skip(2)?;
	let count = reader.varint()?;
	let profiles: Vec<Profile> = (0..count).map_while(|_| profile(reader)).filter(|profile| !profile.name.is_empty()).collect();
	(!profiles.is_empty()).then_some(Event::PartyProfiles { profiles })
}

fn profile(reader: &mut Reader) -> Option<Profile> {
	let mask = reader.u8()?;
	if mask & !PROFILE_FIELDS != 0 {
		return None;
	}
	reader.skip(9)?;
	let name = reader.string()?;
	reader.skip(8)?;
	if mask & PROFILE_CONQUEROR != 0 {
		reader.skip(4)?;
	}
	let gear = reader.u32()?;
	if mask & PROFILE_READY != 0 {
		reader.bit()?;
	}
	reader.bit()?;
	if mask & PROFILE_ORIGIN != 0 {
		reader.skip(2)?;
	}
	if mask & PROFILE_SERVER != 0 {
		reader.skip(2)?;
	}
	reader.skip(1)?;
	let power = reader.u64()?;
	for _ in 0..reader.varint()? {
		reader.skip(5)?;
	}
	if mask & PROFILE_REBIRTH != 0 {
		reader.skip(8)?;
	}
	reader.skip(2)?;
	Some(Profile { name, gear, power })
}

fn uuid(text: &[u8]) -> bool {
	text.iter().enumerate().all(|(index, byte)| if UUID_DASHES.contains(&index) { *byte == b'-' } else { byte.is_ascii_hexdigit() })
}

#[cfg(test)]
mod tests {
	use super::*;

	fn bytes(hex: &str) -> Vec<u8> {
		hex.split_whitespace().map(|byte| u8::from_str_radix(byte, 16).unwrap()).collect()
	}


	fn roster_row(entity: u32, name: &str) -> Vec<u8> {
		let mut row = vec![0x03, 0x04];
		row.extend(entity.to_le_bytes());
		row.extend([0xEF, 0x03, 0xA9, 0x0F, UUID_MARKER]);
		row.extend(b"00000000-0000-4000-8000-000000000001");
		row.extend([0x02, 0x65, 0x01, 0x00, 0x00, 0x00, 0xEF, 0x03, name.len() as u8]);
		row.extend(name.as_bytes());
		row
	}

	#[test]
	fn decodes_back_attack_hit() {
		let body = bytes("04 38 f5 a3 02 06 00 c7 7c e0 26 a8 00 01 02 00 00 01 8b 2f af 41 01 00 00 00 90 4e 2f 01 00");
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 37365, actor: 15943, skill: 11020000, damage: 47, cast: 1, strike: Strike { back: true, ..Strike::default() }, ..Hit::default() })));
	}

	#[test]
	fn decodes_hit_with_additional_strikes() {
		let body = bytes("04 38 91 c1 02 26 00 a1 14 48 f1 ca 00 20 02 0c 00 01 2c 40 46 4f 01 00 00 00 8c 91 01 b1 d1 14 04 97 36 97 36 97 36 97 36 01 00");
		let strike = Strike { perfect: true, hard: true, back: true, additional: true, ..Strike::default() };
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 41105, actor: 2593, skill: 13300040, damage: 338097, cast: 32, strike, ..Hit::default() })));
	}

	#[test]
	fn decodes_layout_four_critical() {
		let body = bytes("04 38 ca d3 02 14 04 ff 09 1a 6a b7 00 0a 03 5b 18 a5 47 01 00 00 00 ce 8a 01 df 47 01 01 00 d0 03");
		assert!(matches!(decode(&body), Some(Event::Hit(Hit { critical: true, damage: 9183, drain: 464, .. }))));
	}

	#[test]
	fn decodes_drain() {
		let body = bytes("04 38 cc a9 01 36 04 fc 29 36 85 c6 00 96 03 80 00 02 23 09 8c 4d 01 00 00 00 9e 55 f4 0e 01 2a 01 00 99 01");
		assert!(matches!(decode(&body), Some(Event::Hit(Hit { damage: 1908, drain: 153, .. }))));
	}

	#[test]
	fn ignores_notice_without_damage() {
		let body = bytes("04 38 9b e0 01 00 00 8b 42 7e 0e f5 00 a2 02 b7 d0 6b 6c 01 00 00");
		assert_eq!(decode(&body), None);
	}

	#[test]
	fn ignores_npc_heal_on_players() {
		let body = bytes("04 38 e8 14 04 00 b6 47 a4 7e 1b 00 01 02 6f a5 18 6a 01 00 00 00 c3 89 01 a8 19 01 00");
		assert_eq!(decode(&body), None);
	}

	#[test]
	fn decodes_direct_heal() {
		let body = bytes("04 38 e8 14 04 00 b6 47 41 9b 0f 01 a6 02 6f a5 18 6a 01 00 00 00 c3 89 01 a8 19 01 00");
		assert_eq!(decode(&body), Some(Event::Heal(Heal { target: 2664, actor: 9142, skill: 17800001, amount: 3240 })));
	}

	#[test]
	fn decodes_hot_tick_and_ignores_pool_announce() {
		assert!(matches!(decode(&bytes("05 38 e8 14 0b 85 0a 9b 01 a3 54 df 65 91 89 01 b6 0b 16 cb 04 01")), Some(Event::Heal(Heal { amount: 1462, .. }))));
		assert_eq!(decode(&bytes("05 38 b6 47 09 85 0a 5f a3 54 df 65 a7 a1 01 16 cb 04 01")), None);
	}

	#[test]
	fn decodes_poison_tick() {
		let body = bytes("05 38 cc a9 01 0a fc 29 94 02 08 54 d6 51 88 03 d7 80 d1 00");
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 21708, actor: 5372, skill: 13730007, damage: 392, dot: true, ..Hit::default() })));
	}

	#[test]
	fn decodes_summon_owner() {
		let body = bytes("41 36 ea bd 02 1f 10 00 00 2c 8e 2c 00 40 02 ff ff ff ff ff ff ff ff 80 75 d5 2a bb 03 00 00 d7 48 00");
		assert_eq!(decode(&body), Some(Event::Spawn { entity: 40682, owner: Some(9303), vitals: None }));
	}

	#[test]
	fn ignores_self_parent() {
		let body = bytes("41 36 9a 8c 01 5f 00 00 00 ff ff ff ff ff ff ff ff 80 75 d5 2a bb 03 00 00 9a 8c 01 00");
		assert_eq!(decode(&body), Some(Event::Spawn { entity: 17946, owner: None, vitals: None }));
	}

	#[test]
	fn decodes_zone_owner() {
		let body = bytes("41 36 f5 88 02 5f 00 00 7b 91 2c 00 40 02 cf 96 05 c7 88 61 89 45 00 d8 d3 45 15 91 67 43 ab a4 01 92 a4 02 92 a4 02 8c 2f 00 00 8c 2f 00 00 00 00 00 00 00 00 00 00 00 00 00 00 f0 c6 02 00 64 00 00 00 f0 49 02 00 01 00 00 00 00 00 00 00 a0 86 01 00 00 00 00 00 70 46 0d 00 01 01 01 11 01 81 96 98 00 ff ff ff ff ff ff ff ff 80 75 d5 2a bb 03 00 00 f5 88 02 01 02 cf 96 05 c7 88 61 89 45 00 d8 d3 45 07 02 06 10 18 00 00");
		assert!(matches!(decode(&body), Some(Event::Spawn { entity: 33909, owner: Some(6160), .. })));
		let body = bytes("41 36 fc a1 02 5f 00 00 00 00 a2 93 2c 00 40 02 00 02 14 c7 00 7d 07 c7 00 2c 57 46 98 e5 02 43 15 5d 01 aa 90 03 aa 90 03 51 28 00 00 51 28 00 00 00 00 00 00 00 00 00 00 00 00 00 00 20 3c 03 00 64 00 00 00 f0 49 02 00 01 00 00 00 00 00 00 00 a0 86 01 00 00 00 00 00 60 01 12 00 01 01 02 11 01 81 96 98 00 ff ff ff ff ff ff ff ff 80 75 d5 2a bb 03 00 00 fc a1 02 01 04 00 02 14 c7 00 7d 07 c7 00 2c 57 46 13 02 eb 53 2c 09 88 13 00 00 00 00 00 00 b4 10 6b 86 a0 01 00 00 fc a1 02 01 63 d5 ea 00 00 02 14 c7 00 7d 07 c7 00 2c 57 46 07 02 01 62 1e 00 00");
		assert!(matches!(decode(&body), Some(Event::Spawn { entity: 37116, owner: Some(7778), .. })));
	}

	#[test]
	fn decodes_kind_4a() {
		let body = bytes("05 38 be e8 01 4a e7 21 4f ec 56 02 44 13 c8 7e 18 00 78 1a ae 00");
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 29758, actor: 4327, skill: 11410040, damage: 19, dot: true, ..Hit::default() })));
	}

	#[test]
	fn decodes_boss_spawn_vitals() {
		let body = bytes("41 36 88 89 01 0c 22 00 00 00 56 1c 23 00 00 02 00 00 00 c0 00 7c 9f c6 00 99 96 47 00 d0 b6 42 00 41 01 80 f7 ba 17 80 f7 ba 17 64 00 00 00 64 00 00 00");
		assert_eq!(decode(&body), Some(Event::Spawn { entity: 17544, owner: None, vitals: Some(Vitals { npc: 2301014, hp: 49_200_000, max: 49_200_000 }) }));
	}

	#[test]
	fn decodes_summon_vitals() {
		let body = bytes("41 36 ea bd 02 1f 10 00 00 00 fe 8e 2c 00 40 02 00 75 0f c7 00 47 06 c7 00 f4 57 46 9a 89 fc 42 cb 59 01 cc c3 0a cc c3 0a fc 1b 00 00 fc 1b 00 00");
		assert!(matches!(decode(&body), Some(Event::Spawn { entity: 40682, vitals: Some(Vitals { npc: 2920190, hp: 172_492, max: 172_492 }), .. })));
	}

	#[test]
	fn decodes_health() {
		assert_eq!(decode(&bytes("00 8d ec 77 02 01 00 e3 5e 00 00 00 00 00 00")), Some(Event::Health { entity: 15340, hp: 24291 }));
		assert_eq!(decode(&bytes("00 8d dd 10 03 01 01 26 0f 00 00 01 00 7d 61 00 00 00 00 00 00")), Some(Event::Health { entity: 2141, hp: 24957 }));
		assert_eq!(decode(&bytes("00 8d b7 32 01 01 03 28 17 02 00")), None);
		assert_eq!(decode(&bytes("00 8d f9 22 02 02 00 b8 5f 00 00 00 00 00 00 07 c8 68 00 00 00 00 00 00")), Some(Event::Health { entity: 4473, hp: 24504 }));
	}

	#[test]
	fn decodes_resurrection() {
		assert_eq!(decode(&bytes("04 38 e9 1d 00 00 e7 79 bb 59 09 01 9d 02 d5 08 a7 67 02 00 00 00 d2 7a 02 00")), Some(Event::Resurrection { target: 3817, actor: 15591 }));
		assert_eq!(decode(&bytes("04 38 e7 79 00 00 e7 79 bb 59 09 01 9d 02 cb 08 a7 67 01 00 00 00 d2 7a 01 00")), None);
	}

	#[test]
	fn decodes_party_profiles() {
		let profile = |name: &str, gear, power| Profile { name: String::from(name), gear, power };
		assert_eq!(decode(&bytes("02 97 72 32 05 00 06 47 72 6f 75 70 65 05 cc 27 09 00 00 03 e9 03 00 00 00 00 00 00 ff 02 03 05 1e 01 ea 03 00 00 00 00 00 00 05 41 6c 70 68 61 22 00 00 00 2d 00 00 00 d6 05 00 00 17 05 d2 10 04 8f 19 01 00 00 00 00 00 00 32 00 00 00 00 00 00 00 01 01 1e 02 ed 03 00 00 00 00 00 00 05 42 72 61 76 6f 15 00 00 00 2d 00 00 00 78 05 00 00 17 05 d2 10 04 be f9 00 00 00 00 00 00 00 37 00 00 00 00 00 00 00 01 01 1e 03 f0 03 00 00 00 00 00 00 07 43 68 61 72 6c 69 65 1e 00 00 00 2d 00 00 00 06 07 00 00 17 05 d2 10 04 a1 49 01 00 00 00 00 00 00 3c 00 00 00 00 00 00 00 01 01 1e 04 f3 03 00 00 00 00 00 00 05 44 65 6c 74 61 10 00 00 00 2d 00 00 00 ab 05 00 00 01 fd 08 d2 10 04 07 18 01 00 00 00 00 00 00 44 00 00 00 00 00 00 00 01 02 00 05 f6 03 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 04 00 00 00 00 00 00 00 00 00 00 00 00 04")), Some(Event::PartyProfiles { profiles: vec![profile("Alpha", 1494, 72079), profile("Bravo", 1400, 63934), profile("Charlie", 1798, 84385), profile("Delta", 1451, 71687)] }));
	}

	#[test]
	fn decodes_avoided_hits() {
		assert_eq!(decode(&bytes("04 38 b5 26 02 00 df b5 01 36 e9 12 00 01 01 20 00 00 23 19 63 07 01 00 00 00 90 4e 01 00")), Some(Event::Avoid { target: 4917, actor: 23263, resisted: false }));
		assert_eq!(decode(&bytes("04 38 d9 7a 00 01 88 89 01 2a 83 1b 00 01 06 74 3c bf 0a 01 00 00 00 90 4e 01 97 1b b7 00 01 00")), Some(Event::Avoid { target: 15705, actor: 17544, resisted: true }));
	}

	#[test]
	fn decodes_buffs() {
		assert_eq!(decode(&bytes("2a 38 fc 29 01 13 59 b1 0c f5 07 b8 0b 00 00 00 00 00 00 2d 57 59 16 a1 01 00 00 fc 29 01")), Some(Event::Buff { target: 5372, caster: 5372, effect: 13350008, duration: 3000 }));
		assert_eq!(decode(&bytes("2b 38 fc 29 13 59 b1 0c f5 07 b8 0b 00 00 00 00 00 00 1b 5a 59 16 a1 01 00 00 fc 29 01")), Some(Event::Buff { target: 5372, caster: 5372, effect: 13350008, duration: 3000 }));
	}

	#[test]
	fn decodes_party_members() {
		assert_eq!(decode(&bytes("0e 92 8b 2b 00")), Some(Event::PartyMember { entity: 5515 }));
		assert_eq!(decode(&bytes("1b 92 e6 7d 00 bf 9c 01 a5 15 00 00 bd 17 00 00 00 00 00 00 00 00 00 00 a6 73 00 00 f0 49 02 00 00")), Some(Event::PartyStatus { entity: 16102, hp: 0, max: 20031 }));
	}

	#[test]
	fn decodes_party_roster() {
		let mut body = bytes("00 92 00 4e 8f 06 00 cb 34 c5 16 00 00 00 00 00 05");
		body.extend(roster_row(5248, "Alpha"));
		body.extend(roster_row(0, "Bravo"));
		body.extend(roster_row(2204, "Charlie"));
		assert_eq!(decode(&body), Some(Event::PartyRoster { members: HashMap::from([(5248, Some(String::from("Alpha"))), (2204, Some(String::from("Charlie")))]) }));
		assert_eq!(decode(&bytes("00 92 00 4e 8f 06 00 cb 34 c5 16 00 00 00 00 00 05")), None);
	}
}