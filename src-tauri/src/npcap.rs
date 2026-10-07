use std::collections::HashSet;
use std::env;
use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString};
use std::ptr;
use std::slice;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use libloading::os::windows::{Library, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR};

use crate::capture::RawPacket;

const ERROR_SIZE: usize = 256;
const SNAPLEN: c_int = 262144;
const TIMEOUT_MILLIS: c_int = 100;
const BUFFER_SIZE: c_int = 16 * 1024 * 1024;
const TIMESTAMP_HOST_PRECISE: c_int = 2;
const NETMASK_UNKNOWN: c_uint = 0xFFFF_FFFF;

#[repr(C)]
struct PcapInterface {
	next: *mut PcapInterface,
	name: *mut c_char,
	description: *mut c_char,
	addresses: *mut c_void,
	flags: c_uint,
}

#[repr(C)]
struct BpfProgram {
	length: c_uint,
	instructions: *mut c_void,
}

#[repr(C)]
struct PacketHeader {
	seconds: i32,
	micros: i32,
	captured: u32,
	length: u32,
}

type FindAllDevices = unsafe extern "C" fn(*mut *mut PcapInterface, *mut c_char) -> c_int;
type FreeAllDevices = unsafe extern "C" fn(*mut PcapInterface);
type Create = unsafe extern "C" fn(*const c_char, *mut c_char) -> *mut c_void;
type SetOption = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type Activate = unsafe extern "C" fn(*mut c_void) -> c_int;
type Compile = unsafe extern "C" fn(*mut c_void, *mut BpfProgram, *const c_char, c_int, c_uint) -> c_int;
type SetFilter = unsafe extern "C" fn(*mut c_void, *mut BpfProgram) -> c_int;
type FreeCode = unsafe extern "C" fn(*mut BpfProgram);
type NextPacket = unsafe extern "C" fn(*mut c_void, *mut *const PacketHeader, *mut *const u8) -> c_int;
type Datalink = unsafe extern "C" fn(*mut c_void) -> c_int;
type GetError = unsafe extern "C" fn(*mut c_void) -> *const c_char;
type Close = unsafe extern "C" fn(*mut c_void);

pub struct Npcap {
	find_all_devices: FindAllDevices,
	free_all_devices: FreeAllDevices,
	create: Create,
	set_snaplen: SetOption,
	set_timeout: SetOption,
	set_buffer_size: SetOption,
	set_immediate_mode: SetOption,
	set_timestamp_type: SetOption,
	activate: Activate,
	compile: Compile,
	set_filter: SetFilter,
	free_code: FreeCode,
	next_packet: NextPacket,
	datalink: Datalink,
	get_error: GetError,
	close: Close,
	_library: Library,
}

struct Capture {
	npcap: Arc<Npcap>,
	handle: *mut c_void,
	linktype: i32,
}

unsafe impl Send for Capture {}

impl Npcap {
	pub fn load() -> Option<Arc<Npcap>> {
		let root = env::var("SystemRoot").unwrap_or_else(|_| String::from("C:\\Windows"));
		unsafe {
			let library = Library::load_with_flags(format!("{root}\\System32\\Npcap\\wpcap.dll"), LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS)
				.or_else(|_| Library::new("wpcap.dll"))
				.ok()?;
			Some(Arc::new(Npcap {
				find_all_devices: *library.get(b"pcap_findalldevs\0").ok()?,
				free_all_devices: *library.get(b"pcap_freealldevs\0").ok()?,
				create: *library.get(b"pcap_create\0").ok()?,
				set_snaplen: *library.get(b"pcap_set_snaplen\0").ok()?,
				set_timeout: *library.get(b"pcap_set_timeout\0").ok()?,
				set_buffer_size: *library.get(b"pcap_set_buffer_size\0").ok()?,
				set_immediate_mode: *library.get(b"pcap_set_immediate_mode\0").ok()?,
				set_timestamp_type: *library.get(b"pcap_set_tstamp_type\0").ok()?,
				activate: *library.get(b"pcap_activate\0").ok()?,
				compile: *library.get(b"pcap_compile\0").ok()?,
				set_filter: *library.get(b"pcap_setfilter\0").ok()?,
				free_code: *library.get(b"pcap_freecode\0").ok()?,
				next_packet: *library.get(b"pcap_next_ex\0").ok()?,
				datalink: *library.get(b"pcap_datalink\0").ok()?,
				get_error: *library.get(b"pcap_geterr\0").ok()?,
				close: *library.get(b"pcap_close\0").ok()?,
				_library: library,
			}))
		}
	}

	fn devices(&self) -> Result<Vec<String>, String> {
		let mut error = [0 as c_char; ERROR_SIZE];
		let mut head = ptr::null_mut();
		if unsafe { (self.find_all_devices)(&mut head, error.as_mut_ptr()) } != 0 {
			return Err(text(error.as_ptr()));
		}
		let mut devices = Vec::new();
		let mut current = head;
		while !current.is_null() {
			let device = unsafe { &*current };
			devices.push(text(device.name));
			current = device.next;
		}
		unsafe { (self.free_all_devices)(head) };
		Ok(devices)
	}

	fn open(self: &Arc<Self>, device: &str, filter: &str) -> Result<Capture, String> {
		let mut error = [0 as c_char; ERROR_SIZE];
		let name = CString::new(device).map_err(|error| error.to_string())?;
		let expression = CString::new(filter).map_err(|error| error.to_string())?;
		let handle = unsafe { (self.create)(name.as_ptr(), error.as_mut_ptr()) };
		if handle.is_null() {
			return Err(text(error.as_ptr()));
		}
		let mut capture = Capture { npcap: self.clone(), handle, linktype: 0 };
		let mut program = BpfProgram { length: 0, instructions: ptr::null_mut() };
		unsafe {
			(self.set_snaplen)(handle, SNAPLEN);
			(self.set_timeout)(handle, TIMEOUT_MILLIS);
			(self.set_buffer_size)(handle, BUFFER_SIZE);
			(self.set_immediate_mode)(handle, 1);
			(self.set_timestamp_type)(handle, TIMESTAMP_HOST_PRECISE);
			if (self.activate)(handle) < 0 || (self.compile)(handle, &mut program, expression.as_ptr(), 1, NETMASK_UNKNOWN) < 0 {
				return Err(capture.error());
			}
			let result = (self.set_filter)(handle, &mut program);
			(self.free_code)(&mut program);
			if result < 0 {
				return Err(capture.error());
			}
			capture.linktype = (self.datalink)(handle);
		}
		Ok(capture)
	}
}

impl Capture {
	fn read(&mut self) -> Result<Option<(u64, &[u8])>, String> {
		let mut header = ptr::null();
		let mut data = ptr::null();
		match unsafe { (self.npcap.next_packet)(self.handle, &mut header, &mut data) } {
			1 => {
				let header = unsafe { &*header };
				let micros = header.seconds as u32 as u64 * 1_000_000 + header.micros as u32 as u64;
				Ok(Some((micros, unsafe { slice::from_raw_parts(data, header.captured as usize) })))
			}
			0 => Ok(None),
			_ => Err(self.error()),
		}
	}

	fn error(&self) -> String {
		text(unsafe { (self.npcap.get_error)(self.handle) })
	}
}

impl Drop for Capture {
	fn drop(&mut self) {
		unsafe { (self.npcap.close)(self.handle) };
	}
}

pub fn listen(npcap: &Arc<Npcap>, filter: &str, sender: &Sender<RawPacket>, active: &Arc<Mutex<HashSet<String>>>) -> Result<usize, String> {
	let mut failure = None;
	for device in npcap.devices()? {
		if active.lock().unwrap().contains(&device) {
			continue;
		}
		let mut capture = match npcap.open(&device, filter) {
			Ok(capture) => capture,
			Err(error) => {
				failure = Some(error);
				continue;
			}
		};
		active.lock().unwrap().insert(device.clone());
		let sender = sender.clone();
		let active = active.clone();
		let linktype = capture.linktype;
		thread::spawn(move || {
			while let Ok(packet) = capture.read() {
				if let Some((micros, data)) = packet {
					if sender.send(RawPacket { micros, linktype, data: data.to_vec() }).is_err() {
						break;
					}
				}
			}
			active.lock().unwrap().remove(&device);
		});
	}
	let count = active.lock().unwrap().len();
	match failure {
		Some(error) if count == 0 => Err(error),
		_ => Ok(count),
	}
}

fn text(pointer: *const c_char) -> String {
	if pointer.is_null() {
		return String::new();
	}
	unsafe { CStr::from_ptr(pointer) }.to_string_lossy().into_owned()
}