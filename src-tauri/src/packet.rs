use std::ops::RangeInclusive;

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
const PARENT_MARKER: [u8; 8] = [0xFF; 8];

const CRITICAL: u64 = 3;
const PLAIN_SWITCH: u64 = 0x04;
const DRAIN_FLAG: u64 = 0x04;
const DEAD_REASON: u8 = 3;
const REVIVE_REASON: u8 = 4;
const HEALTH_STAT: u8 = 0;
const NPC_CODES: RangeInclusive<u32> = 2_000_000..=2_999_999;
const SPAWN_MASKS: [usize; 2] = [4, 2];
const GROUP_HEALS: [u32; 5] = [17_100_000, 17_120_000, 17_290_000, 17_800_000, 18_120_000];
const SELF_HEALS: [u32; 18] = [
	11_260_000, 11_720_000, 11_730_000, 12_260_000, 12_350_000, 12_720_000, 13_260_000, 14_710_000, 16_190_000, 16_770_000, 17_240_000, 17_320_000, 17_720_000, 18_160_000, 18_200_000, 18_420_000, 18_720_000, 19_450_000,
];
const SPIRIT_HEALS: [u32; 2] = [16_190_000, 16_770_000];
const HOT_HEALS: [u32; 4] = [12_350_000, 16_190_000, 17_090_000, 18_120_000];
const GROUP_HOTS: [u32; 2] = [17_090_000, 18_120_000];
const KIND_REMAINING: u8 = 0x02;
const KIND_VALUE: u8 = 0x01;
const KIND_SUFFIX: u8 = 0x08;
const KIND_OTHER: u8 = 0x30;
const KIND_EXTRA: u8 = 0x40;
const HOT_TICK: u8 = 0x0B;
const HOT_LAST_TICK: u8 = 0x0A;

#[derive(Debug, PartialEq)]
pub enum Event {
	Hit(Hit),
	Heal(Heal),
	Character { entity: u64, name: String, own: bool },
	Spawn { entity: u64, owner: Option<u64>, vitals: Option<Vitals> },
	Health { entity: u64, hp: u64 },
	Despawn { entity: u64, dead: bool },
	Combat { entity: u64, active: bool },
	MapChange { map: u32, revive: bool },
}

#[derive(Debug, PartialEq)]
pub struct Hit {
	pub target: u64,
	pub actor: u64,
	pub skill: u32,
	pub damage: u64,
	pub critical: bool,
	pub dot: bool,
	pub drain: u64,
}

#[derive(Debug, PartialEq)]
pub struct Heal {
	pub target: u64,
	pub actor: u64,
	pub skill: u32,
	pub amount: u64,
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
	reader.skip(1)?;
	let kind = reader.varint()?;
	let layout = switch & 0x0F;
	if layout & 0x04 == 0 || skill == 0 {
		return None;
	}
	if layout & 0x02 != 0 {
		reader.skip(1)?;
		reader.varint()?;
		reader.skip(1)?;
	}
	reader.skip(8)?;
	reader.varint()?;
	let amount = reader.varint()?;
	let family = family(skill);
	if switch == PLAIN_SWITCH && (GROUP_HEALS.contains(&family) || (SELF_HEALS.contains(&family) && actor == target)) {
		return Some(Event::Heal(Heal { target, actor, skill, amount }));
	}
	if SPIRIT_HEALS.contains(&family) && actor != target {
		return None;
	}
	let drain = if flag & DRAIN_FLAG != 0 { drain(reader, layout, switch).unwrap_or_default() } else { 0 };
	Some(Event::Hit(Hit { target, actor, skill, damage: amount, critical: kind == CRITICAL, dot: false, drain }))
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
	Some(Event::Hit(Hit { target, actor, skill, damage: remaining?, critical: false, dot: true, drain: 0 }))
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
	let owner = reader
		.seek(&PARENT_MARKER)
		.and_then(|_| {
			reader.skip(8)?;
			reader.varint()
		})
		.filter(|owner| *owner != entity);
	Some(Event::Spawn { entity, owner, vitals })
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
	Some(Event::MapChange { map, revive: reader.u8()? == REVIVE_REASON })
}

#[cfg(test)]
mod tests {
	use super::*;

	fn bytes(hex: &str) -> Vec<u8> {
		hex.split_whitespace().map(|byte| u8::from_str_radix(byte, 16).unwrap()).collect()
	}

	fn hit(target: u64, actor: u64, skill: u32, damage: u64, drain: u64) -> Option<Event> {
		Some(Event::Hit(Hit { target, actor, skill, damage, critical: false, dot: false, drain }))
	}

	#[test]
	fn decodes_back_attack_hit() {
		let body = bytes("04 38 f5 a3 02 06 00 c7 7c e0 26 a8 00 01 02 00 00 01 8b 2f af 41 01 00 00 00 90 4e 2f 01 00");
		assert_eq!(decode(&body), hit(37365, 15943, 11020000, 47, 0));
	}

	#[test]
	fn decodes_hit_with_additional_strikes() {
		let body = bytes("04 38 91 c1 02 26 00 a1 14 48 f1 ca 00 20 02 0c 00 01 2c 40 46 4f 01 00 00 00 8c 91 01 b1 d1 14 04 97 36 97 36 97 36 97 36 01 00");
		assert_eq!(decode(&body), hit(41105, 2593, 13300040, 338097, 0));
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
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 21708, actor: 5372, skill: 13730007, damage: 392, critical: false, dot: true, drain: 0 })));
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
	fn decodes_kind_4a() {
		let body = bytes("05 38 be e8 01 4a e7 21 4f ec 56 02 44 13 c8 7e 18 00 78 1a ae 00");
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 29758, actor: 4327, skill: 11410040, damage: 19, critical: false, dot: true, drain: 0 })));
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
}