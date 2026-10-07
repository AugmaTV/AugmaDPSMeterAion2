use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::frame::count_ticks;
use crate::net::{FlowKey, Segment};

const LOCK_TICKS: usize = 3;
const SWITCH_MICROS: u64 = 1_000_000;
const LIVE_MICROS: u64 = 5_000_000;
const CANDIDATE_MICROS: u64 = 10_000_000;
const BUFFER_MICROS: u64 = 5_000_000;
const GAP_MICROS: u64 = 1_000_000;
const MAX_BUFFER: usize = 4 * 1024 * 1024;
const MAX_PENDING: usize = 512 * 1024;
const MAX_CANDIDATES: usize = 64;
const PROBE_SEGMENTS: usize = 32;
const SEQUENCE_ORIGIN: u64 = 1 << 32;

pub enum Delivery<'a> {
	Start,
	Join,
	Reset,
	Data(&'a [u8]),
}

#[derive(Default)]
pub struct Stream {
	lock: Option<Lock>,
	candidates: HashMap<FlowKey, Candidate>,
}

struct Lock {
	flow: FlowKey,
	next: u64,
	pending: BTreeMap<u64, Vec<u8>>,
	pending_size: usize,
	gap_since: Option<u64>,
	last_seen: u64,
}

struct Candidate {
	ticks: usize,
	last_seen: u64,
	origin: Option<u32>,
	segments: VecDeque<Buffered>,
	size: usize,
}

struct Buffered {
	micros: u64,
	sequence: u32,
	payload: Vec<u8>,
}

impl Stream {
	pub fn push<F: FnMut(Delivery)>(&mut self, micros: u64, segment: &Segment, sink: &mut F) {
		if let Some(lock) = &mut self.lock {
			if lock.flow == segment.flow {
				if !segment.payload.is_empty() {
					lock.last_seen = micros;
				}
				lock.accept(micros, segment.sequence, segment.payload, sink);
				if segment.closing {
					self.lock = None;
				}
				return;
			}
		}
		if self.candidates.len() >= MAX_CANDIDATES {
			self.candidates.retain(|_, candidate| micros.saturating_sub(candidate.last_seen) < CANDIDATE_MICROS);
		}
		if segment.syn {
			self.candidates.insert(segment.flow, Candidate::new(micros, Some(segment.sequence.wrapping_add(1))));
			return;
		}
		if segment.closing {
			self.candidates.remove(&segment.flow);
			return;
		}
		let ticks = count_ticks(segment.payload);
		let mut entry = match self.candidates.entry(segment.flow) {
			Entry::Occupied(entry) => entry,
			Entry::Vacant(entry) if ticks > 0 => entry.insert_entry(Candidate::new(micros, None)),
			Entry::Vacant(_) => return,
		};
		let candidate = entry.get_mut();
		candidate.record(micros, segment.sequence, segment.payload, ticks);
		if candidate.ticks == 0 && candidate.segments.len() >= PROBE_SEGMENTS {
			entry.remove();
			return;
		}
		if candidate.ticks < LOCK_TICKS || !idle_for(&self.lock, micros, SWITCH_MICROS) {
			return;
		}
		let candidate = entry.remove();
		let fresh = candidate.segments.front().is_some_and(|first| candidate.origin == Some(first.sequence));
		let previous = self.lock.as_ref().map(|lock| lock.last_seen);
		let skipped = if fresh { 0 } else { candidate.segments.iter().take_while(|buffered| previous.is_some_and(|last| buffered.micros <= last)).count() };
		let Some(first) = candidate.segments.get(skipped) else {
			return;
		};
		let synced = fresh;
		let lock = self.lock.insert(Lock {
			flow: segment.flow,
			next: SEQUENCE_ORIGIN + first.sequence as u64,
			pending: BTreeMap::new(),
			pending_size: 0,
			gap_since: None,
			last_seen: micros,
		});
		sink(if synced { Delivery::Start } else { Delivery::Join });
		for buffered in candidate.segments.iter().skip(skipped) {
			lock.accept(buffered.micros, buffered.sequence, &buffered.payload, sink);
		}
	}

	pub fn is_live(&self, micros: u64) -> bool {
		!idle_for(&self.lock, micros, LIVE_MICROS)
	}
}

impl Candidate {
	fn new(micros: u64, origin: Option<u32>) -> Self {
		Candidate { ticks: 0, last_seen: micros, origin, segments: VecDeque::new(), size: 0 }
	}

	fn record(&mut self, micros: u64, sequence: u32, payload: &[u8], ticks: usize) {
		self.ticks += ticks;
		self.last_seen = micros;
		if payload.is_empty() {
			return;
		}
		self.size += payload.len();
		self.segments.push_back(Buffered { micros, sequence, payload: payload.to_vec() });
		while let Some(dropped) = self.segments.pop_front_if(|buffered| self.size > MAX_BUFFER || micros.saturating_sub(buffered.micros) > BUFFER_MICROS) {
			self.size -= dropped.payload.len();
			self.origin = None;
		}
	}
}

impl Lock {
	fn accept<F: FnMut(Delivery)>(&mut self, micros: u64, sequence: u32, payload: &[u8], sink: &mut F) {
		if !payload.is_empty() {
			let start = self.unwrap(sequence);
			let end = start + payload.len() as u64;
			if start <= self.next && end > self.next {
				sink(Delivery::Data(&payload[(self.next - start) as usize..]));
				self.next = end;
				self.drain(sink);
			} else if start > self.next {
				let entry = self.pending.entry(start).or_default();
				if payload.len() > entry.len() {
					self.pending_size += payload.len() - entry.len();
					*entry = payload.to_vec();
				}
				self.gap_since.get_or_insert(micros);
			}
		}
		let expired = self.gap_since.is_some_and(|since| micros.saturating_sub(since) >= GAP_MICROS);
		if expired || self.pending_size > MAX_PENDING {
			if let Some(&first) = self.pending.keys().next() {
				self.next = first;
				sink(Delivery::Reset);
				self.drain(sink);
			}
		}
	}

	fn drain<F: FnMut(Delivery)>(&mut self, sink: &mut F) {
		while let Some(entry) = self.pending.first_entry() {
			let start = *entry.key();
			if start > self.next {
				break;
			}
			let data = entry.remove();
			self.pending_size -= data.len();
			let end = start + data.len() as u64;
			if end > self.next {
				sink(Delivery::Data(&data[(self.next - start) as usize..]));
				self.next = end;
			}
		}
		self.gap_since = None;
	}

	fn unwrap(&self, sequence: u32) -> u64 {
		let delta = sequence.wrapping_sub(self.next as u32) as i32 as i64;
		self.next.wrapping_add_signed(delta)
	}
}

fn idle_for(lock: &Option<Lock>, micros: u64, threshold: u64) -> bool {
	lock.as_ref().is_none_or(|lock| micros.saturating_sub(lock.last_seen) >= threshold)
}