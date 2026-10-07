use std::cmp::Reverse;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use augma_dps_meter_lib::capture::Source;
use augma_dps_meter_lib::frame::count_ticks;
use augma_dps_meter_lib::net::{self, FlowKey};
use augma_dps_meter_lib::pcap::PcapWriter;
use augma_dps_meter_lib::pktmon;

const FILTER: &str = "tcp port 13328";
const DEFAULT_SECONDS: u64 = 60;
const REPORT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(200);

#[derive(Default)]
struct FlowStats {
	packets: u64,
	bytes: u64,
	ticks: u64,
}

fn main() {
	let seconds = env::args().nth(1).and_then(|argument| argument.parse().ok()).unwrap_or(DEFAULT_SECONDS);
	let (sender, receiver) = mpsc::channel();
	let source = Source::open(FILTER, &sender).expect("capture impossible : lance ce programme en administrateur");
	let directory = PathBuf::from("captures");
	fs::create_dir_all(&directory).expect("impossible de créer le dossier captures");
	let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or_default();
	println!("Capture via {} pendant {seconds} s...", source.name());
	let mut writers: HashMap<i32, PcapWriter> = HashMap::new();
	let mut flows: HashMap<FlowKey, FlowStats> = HashMap::new();
	let started = Instant::now();
	let mut reported = Instant::now();
	while started.elapsed() < Duration::from_secs(seconds) {
		if let Ok(packet) = receiver.recv_timeout(POLL) {
			if let Some(segment) = net::parse(packet.linktype, &packet.data) {
				let path = directory.join(format!("aion2-{stamp}-{}.pcap", packet.linktype));
				let writer = writers.entry(packet.linktype).or_insert_with(|| PcapWriter::create(&path, packet.linktype).expect("impossible de créer le fichier pcap"));
				writer.write(packet.micros, &packet.data).expect("écriture pcap impossible");
				let stats = flows.entry(segment.flow).or_default();
				stats.packets += 1;
				stats.bytes += segment.payload.len() as u64;
				stats.ticks += count_ticks(segment.payload) as u64;
			}
		}
		if reported.elapsed() >= REPORT {
			let mut active: Vec<_> = flows.iter().filter(|(_, stats)| stats.ticks > 0).collect();
			active.sort_by_key(|(_, stats)| Reverse(stats.ticks));
			for (flow, stats) in active {
				println!("{}:{} -> {}:{}  {} paquets  {} octets  {:.1} ticks/s", Ipv4Addr::from(flow.source), flow.source_port, Ipv4Addr::from(flow.destination), flow.destination_port, stats.packets, stats.bytes, stats.ticks as f64 / REPORT.as_secs_f64());
			}
			println!("{} flux TCP au total", flows.len());
			println!();
			flows.clear();
			reported = Instant::now();
		}
	}
	for writer in writers.values_mut() {
		writer.flush().expect("écriture pcap impossible");
	}
	pktmon::stop();
	println!("Terminé, fichiers dans {}", directory.display());
}