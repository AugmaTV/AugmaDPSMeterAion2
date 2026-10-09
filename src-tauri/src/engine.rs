use crate::frame::Framer;
use crate::meter::{Fight, Meter, Profile, Profiles, Snapshot, Status};
use crate::net;
use crate::packet;
use crate::stream::{Delivery, Stream};

#[derive(Default)]
pub struct Engine {
	stream: Stream,
	framer: Framer,
	meter: Meter,
}

impl Engine {
	pub fn process(&mut self, micros: u64, linktype: i32, data: &[u8]) {
		let Some(segment) = net::parse(linktype, data) else {
			return;
		};
		let Engine { stream, framer, meter } = self;
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
				if let Some(event) = packet::decode(body) {
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