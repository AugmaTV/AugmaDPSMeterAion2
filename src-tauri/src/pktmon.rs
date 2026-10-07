use std::env;
use std::ffi::c_void;
use std::fs;
use std::mem;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_SUCCESS};
use windows_sys::Win32::System::Diagnostics::Etw::{
	CloseTrace, ControlTraceW, EnableTraceEx2, OpenTraceW, ProcessTrace, StartTraceW, TdhGetProperty, CONTROLTRACE_HANDLE, EVENT_CONTROL_CODE_ENABLE_PROVIDER, EVENT_RECORD, EVENT_TRACE_CONTROL_FLUSH, EVENT_TRACE_CONTROL_STOP,
	EVENT_TRACE_LOGFILEW, EVENT_TRACE_PROPERTIES, EVENT_TRACE_REAL_TIME_MODE, PROCESS_TRACE_MODE_EVENT_RECORD, PROCESS_TRACE_MODE_REAL_TIME, PROPERTY_DATA_DESCRIPTOR,
	WNODE_FLAG_TRACED_GUID,
};

use crate::capture::RawPacket;
use crate::net;

const SESSION: &str = "AugmaDPSMeter";
const PROVIDER: GUID = GUID::from_u128(0x4D4F80D9_C8BD_4D73_BB5B_19C90402C5AC);
const PACKET_EVENT: u16 = 160;
const PACKET_ETHERNET: u16 = 1;
const PACKET_WIFI: u16 = 2;
const PACKET_IP: u16 = 3;
const LINKTYPE_ETHERNET: i32 = 1;
const LINKTYPE_RAW: i32 = 12;
const BUFFER_KILOBYTES: u32 = 256;
const MIN_BUFFERS: u32 = 16;
const MAX_BUFFERS: u32 = 128;
const FLUSH_SECONDS: u32 = 1;
const FLUSH_INTERVAL: Duration = Duration::from_millis(100);
const CLOCK_PERFORMANCE_COUNTER: u32 = 1;
const FILETIME_UNIX_MICROS: u64 = 11_644_473_600_000_000;
const NO_WINDOW: u32 = 0x0800_0000;
const LOG_SIZE_MEGABYTES: &str = "1";
const LOG_FILE: &str = "augma-dps-meter.etl";
const INVALID_TRACE: u64 = u64::MAX;
const SNAP_HEADER: [u8; 6] = [0xAA, 0xAA, 0x03, 0x00, 0x00, 0x00];
const ETHERTYPE_IPV4: [u8; 2] = [0x08, 0x00];

static STARTED: AtomicBool = AtomicBool::new(false);
static RUNNING: AtomicBool = AtomicBool::new(false);

#[repr(C)]
struct Properties {
	base: EVENT_TRACE_PROPERTIES,
	name: [u16; 64],
}

struct Consumer {
	sender: Sender<RawPacket>,
	packet_type: Vec<u16>,
	payload_size: Vec<u16>,
	payload: Vec<u16>,
}

impl Properties {
	fn new() -> Self {
		let mut properties: Properties = unsafe { mem::zeroed() };
		properties.base.Wnode.BufferSize = mem::size_of::<Properties>() as u32;
		properties.base.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
		properties.base.Wnode.ClientContext = CLOCK_PERFORMANCE_COUNTER;
		properties.base.BufferSize = BUFFER_KILOBYTES;
		properties.base.MinimumBuffers = MIN_BUFFERS;
		properties.base.MaximumBuffers = MAX_BUFFERS;
		properties.base.FlushTimer = FLUSH_SECONDS;
		properties.base.LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
		properties.base.LoggerNameOffset = mem::offset_of!(Properties, name) as u32;
		properties
	}

	fn as_mut_ptr(&mut self) -> *mut EVENT_TRACE_PROPERTIES {
		self as *mut Properties as *mut EVENT_TRACE_PROPERTIES
	}
}

pub fn start(sender: Sender<RawPacket>) -> Result<(), String> {
	stop();
	let name = wide(SESSION);
	let mut handle = CONTROLTRACE_HANDLE::default();
	let mut properties = Properties::new();
	let mut status = unsafe { StartTraceW(&mut handle, name.as_ptr(), properties.as_mut_ptr()) };
	if status == ERROR_ALREADY_EXISTS {
		stop_session(&name);
		properties = Properties::new();
		status = unsafe { StartTraceW(&mut handle, name.as_ptr(), properties.as_mut_ptr()) };
	}
	check(status)?;
	STARTED.store(true, Ordering::Relaxed);
	if let Err(error) = consume(handle, &name, sender).and_then(|_| capture()) {
		stop();
		return Err(error);
	}
	Ok(())
}

pub fn stop() {
	if STARTED.swap(false, Ordering::Relaxed) {
		let _ = pktmon(&["stop"]);
		stop_session(&wide(SESSION));
		let _ = fs::remove_file(log_file());
	}
}

pub fn running() -> bool {
	RUNNING.load(Ordering::Relaxed)
}

fn consume(handle: CONTROLTRACE_HANDLE, name: &[u16], sender: Sender<RawPacket>) -> Result<(), String> {
	check(unsafe { EnableTraceEx2(handle, &PROVIDER, EVENT_CONTROL_CODE_ENABLE_PROVIDER, u8::MAX, u64::MAX, 0, 0, ptr::null()) })?;
	let consumer = Box::into_raw(Box::new(Consumer { sender, packet_type: wide("PacketType"), payload_size: wide("LoggedPayloadSize"), payload: wide("Payload") }));
	let mut logfile: EVENT_TRACE_LOGFILEW = unsafe { mem::zeroed() };
	logfile.LoggerName = name.as_ptr() as *mut u16;
	logfile.Anonymous1.ProcessTraceMode = PROCESS_TRACE_MODE_REAL_TIME | PROCESS_TRACE_MODE_EVENT_RECORD;
	logfile.Anonymous2.EventRecordCallback = Some(on_event);
	logfile.Context = consumer as *mut c_void;
	let trace = unsafe { OpenTraceW(&mut logfile) };
	if trace.Value == INVALID_TRACE {
		drop(unsafe { Box::from_raw(consumer) });
		return Err(String::from("impossible d'ouvrir la session ETW"));
	}
	let consumer = consumer as usize;
	RUNNING.store(true, Ordering::Relaxed);
	thread::spawn(move || unsafe {
		ProcessTrace(&trace, 1, ptr::null(), ptr::null());
		CloseTrace(trace);
		drop(Box::from_raw(consumer as *mut Consumer));
		RUNNING.store(false, Ordering::Relaxed);
	});
	let name = name.to_vec();
	thread::spawn(move || {
		while unsafe { ControlTraceW(CONTROLTRACE_HANDLE::default(), name.as_ptr(), Properties::new().as_mut_ptr(), EVENT_TRACE_CONTROL_FLUSH) } == ERROR_SUCCESS {
			thread::sleep(FLUSH_INTERVAL);
		}
	});
	Ok(())
}

fn capture() -> Result<(), String> {
	let log = log_file();
	let log = log.to_string_lossy();
	let arguments = ["start", "--capture", "--comp", "nics", "--pkt-size", "0", "--file-name", &log, "--file-size", LOG_SIZE_MEGABYTES];
	if pktmon(&arguments).is_err() {
		let _ = pktmon(&["stop"]);
		pktmon(&arguments)?;
	}
	Ok(())
}

unsafe extern "system" fn on_event(record: *mut EVENT_RECORD) {
	let record = unsafe { &*record };
	if record.EventHeader.EventDescriptor.Id != PACKET_EVENT {
		return;
	}
	let consumer = unsafe { &*(record.UserContext as *const Consumer) };
	let (Some(kind), Some(size)) = (number(record, &consumer.packet_type), number(record, &consumer.payload_size)) else {
		return;
	};
	let mut data = vec![0; size as usize];
	if !property(record, &consumer.payload, &mut data) {
		return;
	}
	let micros = (record.EventHeader.TimeStamp as u64 / 10).saturating_sub(FILETIME_UNIX_MICROS);
	let packet = match kind {
		PACKET_ETHERNET => RawPacket { micros, linktype: LINKTYPE_ETHERNET, data },
		PACKET_IP => RawPacket { micros, linktype: LINKTYPE_RAW, data },
		PACKET_WIFI => match wifi(&data) {
			Some(ip) => RawPacket { micros, linktype: LINKTYPE_RAW, data: ip.to_vec() },
			None => return,
		},
		_ => return,
	};
	if net::parse(packet.linktype, &packet.data).is_some() {
		let _ = consumer.sender.send(packet);
	}
}

fn number(record: &EVENT_RECORD, name: &[u16]) -> Option<u16> {
	let mut buffer = [0; 2];
	property(record, name, &mut buffer).then(|| u16::from_le_bytes(buffer))
}

fn property(record: &EVENT_RECORD, name: &[u16], buffer: &mut [u8]) -> bool {
	let descriptor = PROPERTY_DATA_DESCRIPTOR { PropertyName: name.as_ptr() as u64, ArrayIndex: u32::MAX, Reserved: 0 };
	unsafe { TdhGetProperty(record, 0, ptr::null(), 1, &descriptor, buffer.len() as u32, buffer.as_mut_ptr()) == ERROR_SUCCESS }
}

fn wifi(frame: &[u8]) -> Option<&[u8]> {
	let control = frame.get(..2)?;
	if (control[0] >> 2) & 0x03 != 2 {
		return None;
	}
	let mut header = 24;
	if control[1] & 0x03 == 0x03 {
		header += 6;
	}
	if control[0] & 0x80 != 0 {
		header += 2;
		if control[1] & 0x80 != 0 {
			header += 4;
		}
	}
	let snap = frame.get(header..header + 8)?;
	if snap[..6] != SNAP_HEADER || snap[6..] != ETHERTYPE_IPV4 {
		return None;
	}
	frame.get(header + 8..)
}

fn stop_session(name: &[u16]) {
	let mut properties = Properties::new();
	unsafe { ControlTraceW(CONTROLTRACE_HANDLE::default(), name.as_ptr(), properties.as_mut_ptr(), EVENT_TRACE_CONTROL_STOP) };
}

fn log_file() -> PathBuf {
	env::temp_dir().join(LOG_FILE)
}

fn pktmon(arguments: &[&str]) -> Result<(), String> {
	let root = env::var("SystemRoot").unwrap_or_else(|_| String::from("C:\\Windows"));
	let output = Command::new(format!("{root}\\System32\\PktMon.exe"))
		.args(arguments)
		.stdin(Stdio::null())
		.creation_flags(NO_WINDOW)
		.output()
		.map_err(|error| error.to_string())?;
	if output.status.success() {
		Ok(())
	} else {
		Err(String::from_utf8_lossy(&output.stdout).trim().to_string())
	}
}

fn check(status: u32) -> Result<(), String> {
	if status == ERROR_SUCCESS {
		Ok(())
	} else {
		Err(format!("erreur Windows {status}"))
	}
}

fn wide(text: &str) -> Vec<u16> {
	text.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn strips_decrypted_wifi_header() {
		let mut frame = vec![0x88, 0x42];
		frame.extend([0; 24]);
		frame.extend(SNAP_HEADER);
		frame.extend(ETHERTYPE_IPV4);
		frame.extend([0x45, 0x00]);
		assert_eq!(wifi(&frame), Some(&[0x45, 0x00][..]));
	}

	#[test]
	fn ignores_management_frames() {
		assert_eq!(wifi(&[0x80, 0x00, 0x00, 0x00]), None);
	}
}