use lz4_flex::block::decompress;

use crate::reader::{incomplete_varint, varint};

const TICK: [u8; 3] = [0x0E, 0x00, 0x36];

const TICK_SIZE: usize = 11;
const MIN_TIMESTAMP: u64 = 1_600_000_000_000;
const MAX_TIMESTAMP: u64 = 2_500_000_000_000;
const MAX_FRAME: usize = 65535;
const MAX_BUNDLE: usize = 4 * 1024 * 1024;
const MAX_DEPTH: u32 = 4;
const SYNC_FRAMES: u32 = 2;
const SYNC_WINDOW: usize = 256 * 1024;

#[derive(Default, Clone, Copy)]
pub struct FrameStats {
	pub frames: u64,
	pub bundles: u64,
	pub broken_bundles: u64,
	pub resyncs: u64,
	pub skipped: u64,
}

#[derive(Default)]
pub struct Framer {
	buffer: Vec<u8>,
	synced: bool,
	stats: FrameStats,
}

enum Frame {
	Padding,
	Body(usize, usize),
	Incomplete,
	Invalid,
}

enum Sync {
	Found(usize),
	Pending(usize),
	Missing,
}

impl Framer {
	pub fn start(&mut self) {
		self.buffer.clear();
		self.synced = true;
	}

	pub fn reset(&mut self) {
		self.buffer.clear();
		self.synced = false;
	}

	pub fn stats(&self) -> FrameStats {
		self.stats
	}

	pub fn push<F: FnMut(&[u8])>(&mut self, data: &[u8], sink: &mut F) {
		self.buffer.extend_from_slice(data);
		let mut offset = 0;
		loop {
			if !self.synced {
				match find_sync(&self.buffer[offset..]) {
					Sync::Found(position) => {
						self.stats.skipped += position as u64;
						self.stats.resyncs += 1;
						offset += position;
						self.synced = true;
					}
					Sync::Pending(position) => {
						self.stats.skipped += position as u64;
						offset += position;
						break;
					}
					Sync::Missing => {
						let keep = self.buffer.len().saturating_sub(TICK_SIZE - 1).max(offset);
						self.stats.skipped += (keep - offset) as u64;
						offset = keep;
						break;
					}
				}
			}
			match frame_at(&self.buffer[offset..]) {
				Frame::Padding => offset += 1,
				Frame::Body(start, end) => {
					emit(&self.buffer[offset + start..offset + end], 0, &mut self.stats, sink);
					offset += end;
				}
				Frame::Incomplete => break,
				Frame::Invalid => {
					self.synced = false;
					self.stats.skipped += 1;
					offset += 1;
				}
			}
		}
		self.buffer.drain(..offset);
	}
}

fn is_tick(data: &[u8]) -> bool {
	data.len() >= TICK_SIZE && data[..3] == TICK && (MIN_TIMESTAMP..MAX_TIMESTAMP).contains(&u64::from_le_bytes(data[3..TICK_SIZE].try_into().unwrap_or_default()))
}

pub fn count_ticks(data: &[u8]) -> usize {
	(0..data.len()).filter(|&index| is_tick(&data[index..])).count()
}

fn find_sync(buffer: &[u8]) -> Sync {
	for position in 0..buffer.len().saturating_sub(TICK_SIZE - 1) {
		if !is_tick(&buffer[position..]) {
			continue;
		}
		let mut offset = position;
		let mut frames = 0;
		loop {
			match frame_at(&buffer[offset..]) {
				Frame::Padding => offset += 1,
				Frame::Body(_, end) => {
					offset += end;
					frames += 1;
				}
				Frame::Incomplete if frames >= SYNC_FRAMES && (offset == buffer.len() || offset - position > SYNC_WINDOW) => return Sync::Found(position),
				Frame::Incomplete => return Sync::Pending(position),
				Frame::Invalid => break,
			}
		}
	}
	Sync::Missing
}

fn frame_at(buffer: &[u8]) -> Frame {
	match buffer.first() {
		None => Frame::Incomplete,
		Some(0) => Frame::Padding,
		Some(_) => {
			let Some((value, width)) = varint(buffer) else {
				return if incomplete_varint(buffer) { Frame::Incomplete } else { Frame::Invalid };
			};
			let size = (value as usize).saturating_add(width).saturating_sub(4);
			if size <= width || size > MAX_FRAME {
				Frame::Invalid
			} else if buffer.len() < size {
				Frame::Incomplete
			} else {
				Frame::Body(width, size)
			}
		}
	}
}

fn emit<F: FnMut(&[u8])>(body: &[u8], depth: u32, stats: &mut FrameStats, sink: &mut F) {
	stats.frames += 1;
	let body = match body {
		[0xF0..=0xFE, 0xFF, 0xFF, ..] => &body[1..],
		_ => body,
	};
	let [0xFF, 0xFF, a, b, c, d, compressed @ ..] = body else {
		if body.len() >= 2 {
			sink(body);
		}
		return;
	};
	stats.bundles += 1;
	let size = u32::from_le_bytes([*a, *b, *c, *d]) as usize;
	if depth >= MAX_DEPTH || size > MAX_BUNDLE {
		stats.broken_bundles += 1;
		return;
	}
	match decompress(compressed, size) {
		Ok(plain) if plain.len() == size => walk(&plain, depth + 1, stats, sink),
		_ => stats.broken_bundles += 1,
	}
}

fn walk<F: FnMut(&[u8])>(buffer: &[u8], depth: u32, stats: &mut FrameStats, sink: &mut F) {
	let mut offset = 0;
	while offset < buffer.len() {
		match frame_at(&buffer[offset..]) {
			Frame::Padding => offset += 1,
			Frame::Body(start, end) => {
				emit(&buffer[offset + start..offset + end], depth, stats, sink);
				offset += end;
			}
			_ => {
				stats.broken_bundles += 1;
				return;
			}
		}
	}
}