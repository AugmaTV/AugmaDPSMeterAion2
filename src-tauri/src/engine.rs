use crate::frame::Framer;
use crate::meter::{Meter, Snapshot, Status};
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
			Delivery::Start => framer.start(),
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

	pub fn snapshot(&self, micros: u64) -> Snapshot {
		self.meter.snapshot(if self.stream.is_live(micros) { Status::Live } else { Status::Waiting })
	}
}