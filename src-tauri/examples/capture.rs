use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use augma_dps_meter_lib::frame::count_ticks;
use augma_dps_meter_lib::net::{self, FlowKey};
use augma_dps_meter_lib::npcap::{self, Npcap};
use augma_dps_meter_lib::pcap::PcapWriter;

const DEFAULT_FILTER: &str = "tcp port 13328";
const REPORT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(200);

#[derive(Default)]
struct FlowStats {
	packets: u64,
	bytes: u64,
	ticks: u64,
}

fn main() {
	let filter = env::args().nth(1).unwrap_or_else(|| String::from(DEFAULT_FILTER));
	let npcap = Npcap::load().expect("Npcap introuvable : installe-le depuis https://npcap.com/#download");
	let (sender, receiver) = mpsc::channel();
	let active = Arc::new(Mutex::new(HashSet::new()));
	let opened = npcap::listen(&npcap, &filter, &sender, &active).expect("aucune interface n'a pu être ouverte");
	let directory = PathBuf::from("captures");
	fs::create_dir_all(&directory).expect("impossible de créer le dossier captures");
	let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or_default();
	println!("{opened} interfaces ouvertes, filtre « {filter} ». Ctrl+C pour arrêter.");
	let mut writers: HashMap<i32, PcapWriter> = HashMap::new();
	let mut flows: HashMap<FlowKey, FlowStats> = HashMap::new();
	let mut reported = Instant::now();
	loop {
		if let Ok(packet) = receiver.recv_timeout(POLL) {
			let path = directory.join(format!("aion2-{stamp}-{}.pcap", packet.linktype));
			let writer = writers.entry(packet.linktype).or_insert_with(|| PcapWriter::create(&path, packet.linktype).expect("impossible de créer le fichier pcap"));
			writer.write(packet.micros, &packet.data).expect("écriture pcap impossible");
			if let Some(segment) = net::parse(packet.linktype, &packet.data) {
				let stats = flows.entry(segment.flow).or_default();
				stats.packets += 1;
				stats.bytes += segment.payload.len() as u64;
				stats.ticks += count_ticks(segment.payload) as u64;
			}
		}
		if reported.elapsed() >= REPORT {
			for writer in writers.values_mut() {
				writer.flush().expect("écriture pcap impossible");
			}
			for (flow, stats) in &flows {
				println!("{}:{} -> {}:{}  {} paquets  {} octets  {:.1} ticks/s", Ipv4Addr::from(flow.source), flow.source_port, Ipv4Addr::from(flow.destination), flow.destination_port, stats.packets, stats.bytes, stats.ticks as f64 / REPORT.as_secs_f64());
			}
			println!();
			flows.clear();
			reported = Instant::now();
		}
	}
}