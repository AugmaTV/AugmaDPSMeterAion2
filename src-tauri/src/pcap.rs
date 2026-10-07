use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

const MAGIC: u32 = 0xA1B2_C3D4;
const MAGIC_NANOS: u32 = 0xA1B2_3C4D;
const SNAPLEN: u32 = 65535;

pub struct PcapWriter {
	output: BufWriter<File>,
}

pub struct PcapReader {
	input: BufReader<File>,
	linktype: i32,
	nanos: bool,
}

impl PcapWriter {
	pub fn create(path: &Path, linktype: i32) -> io::Result<PcapWriter> {
		let mut output = BufWriter::new(File::create(path)?);
		output.write_all(&MAGIC.to_le_bytes())?;
		output.write_all(&2u16.to_le_bytes())?;
		output.write_all(&4u16.to_le_bytes())?;
		output.write_all(&[0; 8])?;
		output.write_all(&SNAPLEN.to_le_bytes())?;
		output.write_all(&(linktype as u32).to_le_bytes())?;
		Ok(PcapWriter { output })
	}

	pub fn write(&mut self, micros: u64, data: &[u8]) -> io::Result<()> {
		self.output.write_all(&((micros / 1_000_000) as u32).to_le_bytes())?;
		self.output.write_all(&((micros % 1_000_000) as u32).to_le_bytes())?;
		self.output.write_all(&(data.len() as u32).to_le_bytes())?;
		self.output.write_all(&(data.len() as u32).to_le_bytes())?;
		self.output.write_all(data)
	}

	pub fn flush(&mut self) -> io::Result<()> {
		self.output.flush()
	}
}

impl PcapReader {
	pub fn open(path: &Path) -> io::Result<PcapReader> {
		let mut input = BufReader::new(File::open(path)?);
		let mut header = [0; 24];
		input.read_exact(&mut header)?;
		let magic = u32::from_le_bytes(header[0..4].try_into().unwrap_or_default());
		if magic != MAGIC && magic != MAGIC_NANOS {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "format pcap little-endian attendu"));
		}
		let linktype = u32::from_le_bytes(header[20..24].try_into().unwrap_or_default()) as i32;
		Ok(PcapReader { input, linktype, nanos: magic == MAGIC_NANOS })
	}

	pub fn linktype(&self) -> i32 {
		self.linktype
	}
}

impl Iterator for PcapReader {
	type Item = (u64, Vec<u8>);

	fn next(&mut self) -> Option<Self::Item> {
		let mut header = [0; 16];
		self.input.read_exact(&mut header).ok()?;
		let field = |index: usize| u32::from_le_bytes(header[index * 4..index * 4 + 4].try_into().unwrap_or_default()) as u64;
		let fraction = if self.nanos { field(1) / 1000 } else { field(1) };
		let mut data = vec![0; field(2) as usize];
		self.input.read_exact(&mut data).ok()?;
		Some((field(0) * 1_000_000 + fraction, data))
	}
}