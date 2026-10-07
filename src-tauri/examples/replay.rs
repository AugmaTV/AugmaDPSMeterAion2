use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::path::Path;

use augma_dps_meter_lib::frame::Framer;
use augma_dps_meter_lib::net;
use augma_dps_meter_lib::packet::{self, Event};
use augma_dps_meter_lib::pcap::PcapReader;
use augma_dps_meter_lib::stream::{Delivery, Stream};

const TOP_OPCODES: usize = 25;
const TOP_SKILLS: usize = 12;

#[derive(Default)]
struct Totals {
	damage: u64,
	hits: u64,
	crits: u64,
	dots: u64,
}

fn main() {
	let mut stream = Stream::default();
	let mut framer = Framer::default();
	let mut opcodes: HashMap<[u8; 2], u64> = HashMap::new();
	let mut names: HashMap<u64, String> = HashMap::new();
	let mut owners: HashMap<u64, u64> = HashMap::new();
	let mut actors: HashMap<u64, BTreeMap<u32, Totals>> = HashMap::new();
	for argument in env::args().skip(1) {
		let reader = PcapReader::open(Path::new(&argument)).expect("lecture pcap impossible");
		let linktype = reader.linktype();
		for (micros, data) in reader {
			let Some(segment) = net::parse(linktype, &data) else {
				continue;
			};
			stream.push(micros, &segment, &mut |delivery| match delivery {
				Delivery::Start => framer.start(),
				Delivery::Reset => framer.reset(),
				Delivery::Data(data) => framer.push(data, &mut |body| {
					*opcodes.entry([body[0], body[1]]).or_default() += 1;
					match packet::decode(body) {
						Some(Event::Hit(hit)) => {
							let actor = owners.get(&hit.actor).copied().unwrap_or(hit.actor);
							let totals = actors.entry(actor).or_default().entry(hit.skill - hit.skill % 10_000).or_default();
							totals.damage += hit.damage;
							if hit.dot {
								totals.dots += 1;
							} else {
								totals.hits += 1;
								totals.crits += hit.critical as u64;
							}
						}
						Some(Event::Character { entity, name, .. }) => {
							names.insert(entity, name);
						}
						Some(Event::Spawn { entity, owner: Some(owner) }) => {
							owners.insert(entity, owner);
						}
						_ => {}
					}
				}),
			});
		}
	}
	let stats = framer.stats();
	println!("trames {}  bundles {} (cassés {})  resyncs {}  octets ignorés {}", stats.frames, stats.bundles, stats.broken_bundles, stats.resyncs, stats.skipped);
	let mut sorted: Vec<_> = opcodes.into_iter().collect();
	sorted.sort_by_key(|entry| Reverse(entry.1));
	for (opcode, count) in sorted.into_iter().take(TOP_OPCODES) {
		println!("  {:02X} {:02X}  {count}", opcode[0], opcode[1]);
	}
	let mut players: Vec<_> = actors.into_iter().filter(|(_, skills)| skills.keys().any(|skill| (11_000_000..20_000_000).contains(skill))).collect();
	players.sort_by_key(|(_, skills)| Reverse(skills.values().map(|totals| totals.damage).sum::<u64>()));
	for (actor, skills) in players {
		let damage: u64 = skills.values().map(|totals| totals.damage).sum();
		let hits: u64 = skills.values().map(|totals| totals.hits).sum();
		let crits: u64 = skills.values().map(|totals| totals.crits).sum();
		println!();
		println!("{} (#{actor})  dégâts {damage}  coups {hits}  crits {crits}", names.get(&actor).map(String::as_str).unwrap_or("?"));
		let mut ranked: Vec<_> = skills.into_iter().collect();
		ranked.sort_by_key(|entry| Reverse(entry.1.damage));
		for (skill, totals) in ranked.into_iter().take(TOP_SKILLS) {
			println!("    {skill:>9}  {:>10}  coups {:>5}  crits {:>5}  dot {:>5}", totals.damage, totals.hits, totals.crits, totals.dots);
		}
	}
}