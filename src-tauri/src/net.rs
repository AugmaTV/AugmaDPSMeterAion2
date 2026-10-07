const LINKTYPE_NULL: i32 = 0;
const LINKTYPE_ETHERNET: i32 = 1;
const LINKTYPE_RAW: i32 = 12;
const LINKTYPE_RAW_FILE: i32 = 101;
const LINKTYPE_IPV4: i32 = 228;
const ETHERTYPE_IPV4: u16 = 0x0800;
const ETHERTYPE_VLAN: u16 = 0x8100;
const ETHERTYPE_QINQ: u16 = 0x88A8;
const PROTOCOL_TCP: u8 = 6;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowKey {
	pub source: [u8; 4],
	pub source_port: u16,
	pub destination: [u8; 4],
	pub destination_port: u16,
}

pub struct Segment<'a> {
	pub flow: FlowKey,
	pub sequence: u32,
	pub syn: bool,
	pub closing: bool,
	pub payload: &'a [u8],
}

pub fn parse(linktype: i32, data: &[u8]) -> Option<Segment<'_>> {
	let ip = match linktype {
		LINKTYPE_NULL => data.get(4..)?,
		LINKTYPE_ETHERNET => ethernet(data)?,
		LINKTYPE_RAW | LINKTYPE_RAW_FILE | LINKTYPE_IPV4 => data,
		_ => return None,
	};
	ipv4(ip)
}

fn ethernet(data: &[u8]) -> Option<&[u8]> {
	let mut offset = 12;
	let mut ethertype = u16_be(data, offset)?;
	while ethertype == ETHERTYPE_VLAN || ethertype == ETHERTYPE_QINQ {
		offset += 4;
		ethertype = u16_be(data, offset)?;
	}
	if ethertype != ETHERTYPE_IPV4 {
		return None;
	}
	data.get(offset + 2..)
}

fn ipv4(data: &[u8]) -> Option<Segment<'_>> {
	let first = *data.first()?;
	let header = (first & 0x0F) as usize * 4;
	let total = u16_be(data, 2)? as usize;
	let fragment = u16_be(data, 6)?;
	if first >> 4 != 4 || header < 20 || *data.get(9)? != PROTOCOL_TCP || fragment & 0x3FFF != 0 {
		return None;
	}
	let end = if total == 0 { data.len() } else { total.min(data.len()) };
	let tcp = data.get(header..end)?;
	let offset = (*tcp.get(12)? >> 4) as usize * 4;
	let flags = *tcp.get(13)?;
	Some(Segment {
		flow: FlowKey {
			source: data[12..16].try_into().ok()?,
			source_port: u16_be(tcp, 0)?,
			destination: data[16..20].try_into().ok()?,
			destination_port: u16_be(tcp, 2)?,
		},
		sequence: u32::from_be_bytes(tcp[4..8].try_into().ok()?),
		syn: flags & 0x02 != 0,
		closing: flags & 0x05 != 0,
		payload: tcp.get(offset..)?,
	})
}

fn u16_be(data: &[u8], offset: usize) -> Option<u16> {
	Some(u16::from_be_bytes(data.get(offset..offset + 2)?.try_into().ok()?))
}