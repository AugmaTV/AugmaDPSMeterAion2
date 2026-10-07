use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::npcap::{self, Npcap};
use crate::pktmon;

pub struct RawPacket {
	pub micros: u64,
	pub linktype: i32,
	pub data: Vec<u8>,
}

pub enum Source {
	Npcap { npcap: Arc<Npcap>, active: Arc<Mutex<HashSet<String>>> },
	Pktmon,
}

impl Source {
	pub fn open(filter: &str, sender: &Sender<RawPacket>) -> Result<Source, String> {
		if let Some(npcap) = Npcap::load() {
			let active = Arc::new(Mutex::new(HashSet::new()));
			if npcap::listen(&npcap, filter, sender, &active).is_ok_and(|count| count > 0) {
				return Ok(Source::Npcap { npcap, active });
			}
		}
		pktmon::start(sender.clone()).map(|_| Source::Pktmon)
	}

	pub fn refresh(&self, filter: &str, sender: &Sender<RawPacket>) {
		match self {
			Source::Npcap { npcap, active } => {
				let _ = npcap::listen(npcap, filter, sender, active);
			}
			Source::Pktmon => {
				if !pktmon::running() {
					let _ = pktmon::start(sender.clone());
				}
			}
		}
	}

	pub fn name(&self) -> &'static str {
		match self {
			Source::Npcap { .. } => "Npcap",
			Source::Pktmon => "pktmon",
		}
	}
}