const MAX_VARINT: usize = 10;

pub struct Reader<'a> {
	data: &'a [u8],
	offset: usize,
	bits: u8,
	left: u8,
}

impl<'a> Reader<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Reader { data, offset: 0, bits: 0, left: 0 }
	}

	pub fn bit(&mut self) -> Option<bool> {
		if self.left == 0 {
			self.bits = self.u8()?;
			self.left = 8;
		}
		let bit = self.bits & 1 == 1;
		self.bits >>= 1;
		self.left -= 1;
		Some(bit)
	}

	pub fn u8(&mut self) -> Option<u8> {
		Some(self.take(1)?[0])
	}

	pub fn u32(&mut self) -> Option<u32> {
		Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
	}

	pub fn u64(&mut self) -> Option<u64> {
		Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
	}

	pub fn varint(&mut self) -> Option<u64> {
		let (value, width) = varint(&self.data[self.offset..])?;
		self.offset += width;
		Some(value)
	}

	pub fn string(&mut self) -> Option<String> {
		let length = self.u8()? as usize;
		String::from_utf8(self.take(length)?.to_vec()).ok()
	}

	pub fn skip(&mut self, count: usize) -> Option<()> {
		self.take(count).map(|_| ())
	}

	pub fn seek(&mut self, pattern: &[u8]) -> Option<()> {
		let position = self.data[self.offset..].windows(pattern.len()).position(|window| window == pattern)?;
		self.offset += position + pattern.len();
		Some(())
	}

	pub fn rest(&self) -> &'a [u8] {
		&self.data[self.offset..]
	}

	fn take(&mut self, count: usize) -> Option<&'a [u8]> {
		let slice = self.data.get(self.offset..self.offset.checked_add(count)?)?;
		self.offset += count;
		Some(slice)
	}
}

pub fn varint(data: &[u8]) -> Option<(u64, usize)> {
	let mut value = 0;
	for (index, byte) in data.iter().take(MAX_VARINT).enumerate() {
		value |= ((byte & 0x7F) as u64) << (index * 7);
		if byte & 0x80 == 0 {
			return Some((value, index + 1));
		}
	}
	None
}

pub fn incomplete_varint(data: &[u8]) -> bool {
	data.len() < MAX_VARINT && data.iter().all(|byte| byte & 0x80 != 0)
}