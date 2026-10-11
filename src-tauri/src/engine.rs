use std::collections::BTreeMap;

use serde::Serialize;

use crate::frame::Framer;
use crate::meter::{Fight, Meter, Profile, Profiles, Snapshot, Status};
use crate::net;
use crate::packet::{self, Event};
use crate::stream::{Delivery, Stream};

const WATCHED: [([u8; 2], &str); 5] = [([0x33, 0x36], "character"), ([0x11, 0x56], "equipment"), ([0x00, 0x51], "skills"), ([0x26, 0xE2], "daevanion"), ([0x00, 0x90], "pets")];

#[derive(Serialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
	pub count: u32,
	pub size: usize,
	pub decoded: bool,
	pub items: usize,
}

#[derive(Default)]
pub struct Engine {
	stream: Stream,
	framer: Framer,
	meter: Meter,
	diagnostics: BTreeMap<String, Diagnostic>,
	diagnostics_changed: bool,
}

fn describe(event: &Option<Event>) -> (bool, usize) {
	match event {
		Some(Event::Character { equipment, .. }) => (true, equipment.len()),
		Some(Event::OwnEquipment { equipment }) => (true, equipment.len()),
		Some(Event::SkillLevels { levels }) => (true, levels.len()),
		Some(Event::Daevanion { boards }) => (true, boards.len()),
		Some(Event::Perception { species, pets }) => (true, species.len() + pets.len()),
		_ => (false, 0),
	}
}

impl Engine {
	pub fn process(&mut self, micros: u64, linktype: i32, data: &[u8]) {
		let Some(segment) = net::parse(linktype, data) else {
			return;
		};
		let Engine { stream, framer, meter, diagnostics, diagnostics_changed } = self;
		stream.push(micros, &segment, &mut |delivery| match delivery {
			Delivery::Start => {
				framer.start();
				meter.reconnect(micros);
			}
			Delivery::Join => {
				framer.reset();
				meter.reconnect(micros);
			}
			Delivery::Reset => framer.reset(),
			Delivery::Data(data) => framer.push(data, &mut |body| {
				let event = packet::decode(body);
				if let Some((_, key)) = WATCHED.iter().find(|(code, _)| body[..2] == code[..]) {
					let (decoded, items) = describe(&event);
					let entry = diagnostics.entry((*key).to_string()).or_default();
					let next = Diagnostic { count: entry.count + 1, size: body.len(), decoded, items };
					if *entry != next {
						*entry = next;
						*diagnostics_changed = true;
					}
				}
				if let Some(event) = event {
					meter.apply(micros, event);
				}
			}),
		});
	}

	pub fn reset(&mut self) {
		self.meter.reset();
	}

	pub fn set_party_only(&mut self, enabled: bool) {
		self.meter.set_party_only(enabled);
	}

	pub fn set_dungeon(&mut self, enabled: bool) {
		self.meter.set_dungeon(enabled);
	}

	pub fn closed(&mut self) -> Vec<Vec<Fight>> {
		self.meter.closed()
	}

	pub fn session(&self) -> Vec<Fight> {
		self.meter.session()
	}

	pub fn diagnostics(&self) -> &BTreeMap<String, Diagnostic> {
		&self.diagnostics
	}

	pub fn take_diagnostics_change(&mut self) -> bool {
		std::mem::take(&mut self.diagnostics_changed)
	}

	pub fn profile(&self) -> Option<Profile> {
		self.meter.profile()
	}

	pub fn profiles(&self) -> &Profiles {
		self.meter.profiles()
	}

	pub fn restore_profiles(&mut self, profiles: Profiles) {
		self.meter.restore_profiles(profiles);
	}

	pub fn take_profile_change(&mut self) -> bool {
		self.meter.take_profile_change()
	}

	pub fn snapshot(&self, micros: u64) -> Snapshot {
		self.meter.snapshot(if self.stream.is_live(micros) { Status::Live } else { Status::Waiting })
	}
}