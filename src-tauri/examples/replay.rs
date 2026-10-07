use std::cmp::Reverse;
use std::collections::HashMap;
use std::env;
use std::path::Path;

use augma_dps_meter_lib::engine::Engine;
use augma_dps_meter_lib::frame::Framer;
use augma_dps_meter_lib::meter::Snapshot;
use augma_dps_meter_lib::net;
use augma_dps_meter_lib::pcap::PcapReader;
use augma_dps_meter_lib::stream::{Delivery, Stream};

const TOP_OPCODES: usize = 25;
const TOP_PLAYERS: usize = 6;
const SAMPLE_MICROS: u64 = 250_000;

fn main() {
	let mut stream = Stream::default();
	let mut framer = Framer::default();
	let mut opcodes: HashMap<[u8; 2], u64> = HashMap::new();
	for argument in env::args().skip(1) {
		let reader = PcapReader::open(Path::new(&argument)).expect("lecture pcap impossible");
		let linktype = reader.linktype();
		let mut engine = Engine::default();
		let mut origin = None;
		let mut sampled = 0;
		let mut current: Option<(u64, Snapshot)> = None;
		println!("== {argument}");
		for (micros, data) in reader {
			let origin = *origin.get_or_insert(micros);
			engine.process(micros, linktype, &data);
			if let Some(segment) = net::parse(linktype, &data) {
				stream.push(micros, &segment, &mut |delivery| match delivery {
					Delivery::Start => framer.start(),
					Delivery::Join | Delivery::Reset => framer.reset(),
					Delivery::Data(data) => framer.push(data, &mut |body| *opcodes.entry([body[0], body[1]]).or_default() += 1),
				});
			}
			if micros - sampled < SAMPLE_MICROS {
				continue;
			}
			sampled = micros;
			let snapshot = engine.snapshot(micros);
			if let Some((at, previous)) = &current {
				if snapshot.duration < previous.duration || snapshot.total < previous.total {
					report(*at - origin, previous);
				}
			}
			if snapshot.total > 0 {
				let changed = current.as_ref().is_none_or(|(_, previous)| previous.duration != snapshot.duration);
				let at = if changed { micros } else { current.as_ref().map_or(micros, |(at, _)| *at) };
				current = Some((at, snapshot));
			}
		}
		if let (Some((at, previous)), Some(origin)) = (&current, origin) {
			report(*at - origin, previous);
		}
	}
	let stats = framer.stats();
	println!();
	println!("trames {}  bundles {} (cassés {})  resyncs {}  octets ignorés {}", stats.frames, stats.bundles, stats.broken_bundles, stats.resyncs, stats.skipped);
	let mut sorted: Vec<_> = opcodes.into_iter().collect();
	sorted.sort_by_key(|entry| Reverse(entry.1));
	for (opcode, count) in sorted.into_iter().take(TOP_OPCODES) {
		println!("  {:02X} {:02X}  {count}", opcode[0], opcode[1]);
	}
}

fn report(at: u64, snapshot: &Snapshot) {
	let end = at as f64 / 1_000_000.0;
	let start = end - snapshot.duration as f64 / 1000.0;
	let boss = snapshot.boss.as_ref().map(|boss| format!("  boss {} {}/{}{}{}", boss.npc.unwrap_or_default(), boss.hp, boss.max, if boss.estimated { "~" } else { "" }, if boss.dead { " mort" } else { "" })).unwrap_or_default();
	println!("  {start:7.1}–{end:7.1} s  dégâts {}  soins {}{boss}", snapshot.total, snapshot.total_healing);
	for player in snapshot.players.iter().take(TOP_PLAYERS) {
		println!("      {:<16} classe {}  dégâts {:>10}  soins {:>9}", player.name.as_deref().unwrap_or("?"), player.class, player.damage, player.healing);
	}
}