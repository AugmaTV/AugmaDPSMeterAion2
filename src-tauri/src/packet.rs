use crate::reader::Reader;

const HIT: [u8; 2] = [0x04, 0x38];
const DOT: [u8; 2] = [0x05, 0x38];
const OWN_CHARACTER: [u8; 2] = [0x33, 0x36];
const OTHER_CHARACTER: [u8; 2] = [0x45, 0x36];
const SPAWN: [u8; 2] = [0x41, 0x36];
const MAP_CHANGE: [u8; 2] = [0x21, 0x36];
const PARENT_MARKER: [u8; 8] = [0xFF; 8];

const CRITICAL: u64 = 3;
const DOT_DAMAGE: u8 = 0x02;
const DOT_DAMAGE_STACKED: u8 = 0x0A;

#[derive(Debug, PartialEq)]
pub enum Event {
	Hit(Hit),
	Character { entity: u64, name: String, own: bool },
	Spawn { entity: u64, owner: Option<u64> },
	MapChange,
}

#[derive(Debug, PartialEq)]
pub struct Hit {
	pub target: u64,
	pub actor: u64,
	pub skill: u32,
	pub damage: u64,
	pub critical: bool,
	pub dot: bool,
}

pub fn decode(body: &[u8]) -> Option<Event> {
	let mut reader = Reader::new(body.get(2..)?);
	match [body[0], body[1]] {
		HIT => hit(&mut reader),
		DOT => dot(&mut reader),
		OWN_CHARACTER => character(&mut reader, true),
		OTHER_CHARACTER => character(&mut reader, false),
		SPAWN => spawn(&mut reader),
		MAP_CHANGE => Some(Event::MapChange),
		_ => None,
	}
}

fn hit(reader: &mut Reader) -> Option<Event> {
	let target = reader.varint()?;
	let switch = reader.varint()?;
	reader.varint()?;
	let actor = reader.varint()?;
	let skill = reader.u32()?;
	reader.skip(1)?;
	let kind = reader.varint()?;
	let layout = switch & 0x0F;
	if layout & 0x04 == 0 {
		return None;
	}
	if layout & 0x02 != 0 {
		reader.skip(1)?;
		reader.varint()?;
		reader.skip(1)?;
	}
	reader.skip(8)?;
	reader.varint()?;
	let damage = reader.varint()?;
	Some(Event::Hit(Hit { target, actor, skill, damage, critical: kind == CRITICAL, dot: false }))
}

fn dot(reader: &mut Reader) -> Option<Event> {
	let target = reader.varint()?;
	let kind = reader.u8()?;
	let actor = reader.varint()?;
	reader.varint()?;
	let effect = reader.u32()?;
	let damage = reader.varint()?;
	if kind != DOT_DAMAGE && kind != DOT_DAMAGE_STACKED || actor == target {
		return None;
	}
	Some(Event::Hit(Hit { target, actor, skill: effect / 100, damage, critical: false, dot: true }))
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
	let owner = reader
		.seek(&PARENT_MARKER)
		.and_then(|_| {
			reader.skip(8)?;
			reader.varint()
		})
		.filter(|owner| *owner != entity);
	Some(Event::Spawn { entity, owner })
}

#[cfg(test)]
mod tests {
	use super::*;

	fn bytes(hex: &str) -> Vec<u8> {
		hex.split_whitespace().map(|byte| u8::from_str_radix(byte, 16).unwrap()).collect()
	}

	#[test]
	fn decodes_back_attack_hit() {
		let body = bytes("04 38 f5 a3 02 06 00 c7 7c e0 26 a8 00 01 02 00 00 01 8b 2f af 41 01 00 00 00 90 4e 2f 01 00");
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 37365, actor: 15943, skill: 11020000, damage: 47, critical: false, dot: false })));
	}

	#[test]
	fn decodes_hit_with_additional_strikes() {
		let body = bytes("04 38 91 c1 02 26 00 a1 14 48 f1 ca 00 20 02 0c 00 01 2c 40 46 4f 01 00 00 00 8c 91 01 b1 d1 14 04 97 36 97 36 97 36 97 36 01 00");
		assert_eq!(decode(&body), Some(Event::Hit(Hit { target: 41105, actor: 2593, skill: 13300040, damage: 338097, critical: false, dot: false })));
	}

	#[test]
	fn decodes_layout_four_critical() {
		let body = bytes("04 38 ca d3 02 14 04 ff 09 1a 6a b7 00 0a 03 5b 18 a5 47 01 00 00 00 ce 8a 01 df 47 01 01 00 d0 03");
		assert!(matches!(decode(&body), Some(Event::Hit(Hit { critical: true, damage: 9183, .. }))));
	}

	#[test]
	fn ignores_notice_without_damage() {
		let body = bytes("04 38 9b e0 01 00 00 8b 42 7e 0e f5 00 a2 02 b7 d0 6b 6c 01 00 00");
		assert_eq!(decode(&body), None);
	}

	#[test]
	fn decodes_summon_owner() {
		let body = bytes("41 36 ea bd 02 1f 10 00 00 2c 8e 2c 00 40 02 ff ff ff ff ff ff ff ff 80 75 d5 2a bb 03 00 00 d7 48 00");
		assert_eq!(decode(&body), Some(Event::Spawn { entity: 40682, owner: Some(9303) }));
	}

	#[test]
	fn ignores_self_parent() {
		let body = bytes("41 36 9a 8c 01 5f 00 00 00 ff ff ff ff ff ff ff ff 80 75 d5 2a bb 03 00 00 9a 8c 01 00");
		assert_eq!(decode(&body), Some(Event::Spawn { entity: 17946, owner: None }));
	}
}