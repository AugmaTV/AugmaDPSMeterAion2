pub mod capture;
pub mod engine;
pub mod frame;
pub mod meter;
pub mod net;
pub mod npcap;
pub mod packet;
pub mod pcap;
pub mod pktmon;
mod reader;
pub mod stream;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{async_runtime, command, generate_context, generate_handler, AppHandle, Builder, Emitter, Manager, Result, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::StateFlags;

use crate::capture::Source;
use crate::engine::Engine;
use crate::meter::{Meter, Status};

const WINDOW: &str = "main";
const SPLASH: &str = "splash";
const OVERLAY: &str = "overlay";
const LOCK_SHORTCUT: &str = "ctrl+shift+l";
const FILTER: &str = "tcp";
const REFRESH: Duration = Duration::from_millis(500);
const RESCAN: Duration = Duration::from_secs(5);
const RETRY: Duration = Duration::from_secs(3);
const UPDATE_TIMEOUT: Duration = Duration::from_secs(8);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);

static RESET: AtomicBool = AtomicBool::new(false);
static LOCKED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone)]
struct OverlayState {
	open: bool,
	locked: bool,
}

#[derive(Serialize, Clone)]
#[serde(tag = "stage", rename_all = "kebab-case")]
enum UpdateState {
	Checking,
	Downloading { version: String, progress: Option<u64> },
	Installing { version: String },
}

#[command]
fn reset() {
	RESET.store(true, Ordering::Relaxed);
}

#[command]
fn overlay(app: AppHandle) -> OverlayState {
	OverlayState { open: app.get_webview_window(OVERLAY).is_some(), locked: LOCKED.load(Ordering::Relaxed) }
}

#[command]
async fn toggle_overlay(app: AppHandle) -> Result<()> {
	if let Some(window) = app.get_webview_window(OVERLAY) {
		return window.close();
	}
	LOCKED.store(false, Ordering::Relaxed);
	WebviewWindowBuilder::new(&app, OVERLAY, WebviewUrl::default())
		.title("Augma DPS Overlay")
		.inner_size(300.0, 220.0)
		.min_inner_size(200.0, 100.0)
		.decorations(false)
		.transparent(true)
		.shadow(false)
		.always_on_top(true)
		.skip_taskbar(true)
		.focused(false)
		.build()?;
	let _ = app.global_shortcut().register(LOCK_SHORTCUT);
	app.emit("overlay", OverlayState { open: true, locked: false })
}

#[command]
fn lock_overlay(app: AppHandle, locked: bool) {
	set_locked(&app, locked);
}

pub fn run() {
	Builder::default()
		.plugin(tauri_plugin_single_instance::init(|app, _, _| {
			if let Some(window) = app.get_webview_window(WINDOW) {
				let _ = window.unminimize();
				let _ = window.set_focus();
			}
		}))
		.plugin(tauri_plugin_window_state::Builder::new().with_state_flags(StateFlags::POSITION | StateFlags::SIZE | StateFlags::MAXIMIZED).with_denylist(&[SPLASH]).build())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.plugin(
			tauri_plugin_global_shortcut::Builder::new()
				.with_handler(|app, _, event| {
					if event.state == ShortcutState::Pressed {
						set_locked(app, !LOCKED.load(Ordering::Relaxed));
					}
				})
				.build(),
		)
		.invoke_handler(generate_handler![reset, overlay, toggle_overlay, lock_overlay])
		.on_window_event(|window, event| {
			if !matches!(event, WindowEvent::Destroyed) {
				return;
			}
			let app = window.app_handle();
			if window.label() == WINDOW {
				app.exit(0);
			} else if window.label() == OVERLAY {
				LOCKED.store(false, Ordering::Relaxed);
				let _ = app.global_shortcut().unregister(LOCK_SHORTCUT);
				let _ = app.emit("overlay", OverlayState { open: false, locked: false });
			}
		})
		.setup(|app| {
			let handle = app.handle();
			capture(handle.clone());
			update(handle.clone());
			Ok(())
		})
		.build(generate_context!())
		.expect("error while running tauri application")
		.run(|_, event| {
			if let RunEvent::Exit = event {
				pktmon::stop();
			}
		});
}

fn set_locked(app: &AppHandle, locked: bool) {
	LOCKED.store(locked, Ordering::Relaxed);
	if let Some(window) = app.get_webview_window(OVERLAY) {
		let _ = window.set_ignore_cursor_events(locked);
	}
	let _ = app.emit("overlay", OverlayState { open: app.get_webview_window(OVERLAY).is_some(), locked });
}

fn capture(app: AppHandle) {
	thread::spawn(move || {
		let (sender, receiver) = mpsc::channel();
		let source = loop {
			match Source::open(FILTER, &sender) {
				Ok(source) => break source,
				Err(_) => {
					let _ = app.emit("snapshot", Meter::default().snapshot(Status::Unavailable));
					thread::sleep(RETRY);
				}
			}
		};
		let mut engine = Engine::default();
		let mut scanned = Instant::now();
		let mut emitted = Instant::now();
		loop {
			if scanned.elapsed() >= RESCAN {
				source.refresh(FILTER, &sender);
				scanned = Instant::now();
			}
			if let Ok(packet) = receiver.recv_timeout(REFRESH) {
				engine.process(packet.micros, packet.linktype, &packet.data);
			}
			if RESET.swap(false, Ordering::Relaxed) {
				engine.reset();
			}
			if emitted.elapsed() >= REFRESH {
				let _ = app.emit("snapshot", engine.snapshot(now()));
				emitted = Instant::now();
			}
		}
	});
}

fn update(app: AppHandle) {
	async_runtime::spawn(async move {
		let _ = app.emit("update", UpdateState::Checking);
		let update = match app.updater_builder().restart_after_install(false).timeout(UPDATE_TIMEOUT).build() {
			Ok(updater) => updater.check().await.ok().flatten(),
			Err(_) => None,
		};
		if let Some(mut update) = update {
			update.timeout = Some(DOWNLOAD_TIMEOUT);
			let version = update.version.clone();
			let _ = app.emit("update", UpdateState::Downloading { version: version.clone(), progress: None });
			let mut downloaded = 0;
			let mut reported = None;
			let download = update.download(
				|chunk, total| {
					downloaded += chunk as u64;
					let progress = total.map(|total| downloaded * 100 / total.max(1));
					if progress != reported {
						reported = progress;
						let _ = app.emit("update", UpdateState::Downloading { version: version.clone(), progress });
					}
				},
				|| {},
			);
			if let Ok(bytes) = download.await {
				let _ = app.emit("update", UpdateState::Installing { version: version.clone() });
				pktmon::stop();
				let _ = update.install(bytes);
			}
		}
		reveal(&app);
	});
}

fn reveal(app: &AppHandle) {
	if let Some(window) = app.get_webview_window(WINDOW) {
		let _ = window.show();
		let _ = window.set_focus();
	}
	if let Some(splash) = app.get_webview_window(SPLASH) {
		let _ = splash.close();
	}
}

fn now() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_micros() as u64).unwrap_or_default()
}