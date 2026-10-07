pub mod capture;
mod engine;
pub mod frame;
mod meter;
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

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{async_runtime, command, generate_context, generate_handler, AppHandle, Builder, Emitter, Manager, Result, RunEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::StateFlags;

use crate::capture::Source;
use crate::engine::Engine;
use crate::meter::{Meter, Status};

const OVERLAY: &str = "overlay";
const FILTER: &str = "tcp";
const LOCK_SHORTCUT: &str = "ctrl+shift+l";
const REFRESH: Duration = Duration::from_millis(500);
const RESCAN: Duration = Duration::from_secs(5);
const RETRY: Duration = Duration::from_secs(3);

static LOCKED: AtomicBool = AtomicBool::new(true);
static RESET: AtomicBool = AtomicBool::new(false);

#[command]
fn locked() -> bool {
	LOCKED.load(Ordering::Relaxed)
}

#[command]
fn lock(app: AppHandle) {
	set_locked(&app, true);
}

#[command]
fn reset() {
	RESET.store(true, Ordering::Relaxed);
}

pub fn run() {
	Builder::default()
		.plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
		.plugin(tauri_plugin_window_state::Builder::new().with_state_flags(StateFlags::POSITION | StateFlags::SIZE).build())
		.plugin(tauri_plugin_updater::Builder::new().build())
		.plugin(
			tauri_plugin_global_shortcut::Builder::new()
				.with_handler(|app, _, event| {
					if event.state == ShortcutState::Pressed {
						toggle_lock(app);
					}
				})
				.build(),
		)
		.invoke_handler(generate_handler![locked, lock, reset])
		.setup(|app| {
			let handle = app.handle();
			let _ = handle.global_shortcut().register(LOCK_SHORTCUT);
			set_locked(handle, true);
			tray(handle)?;
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

fn tray(app: &AppHandle) -> Result<()> {
	let toggle = MenuItem::with_id(app, "toggle", "Verrouiller / déverrouiller (Ctrl+Shift+L)", true, None::<&str>)?;
	let reset = MenuItem::with_id(app, "reset", "Réinitialiser", true, None::<&str>)?;
	let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
	TrayIconBuilder::new()
		.icon(app.default_window_icon().expect("icône manquante").clone())
		.tooltip("Augma DPS Meter")
		.menu(&Menu::with_items(app, &[&toggle, &reset, &quit])?)
		.on_menu_event(|app, event| match event.id.as_ref() {
			"toggle" => toggle_lock(app),
			"reset" => RESET.store(true, Ordering::Relaxed),
			"quit" => app.exit(0),
			_ => {}
		})
		.build(app)?;
	Ok(())
}

fn toggle_lock(app: &AppHandle) {
	set_locked(app, !LOCKED.load(Ordering::Relaxed));
}

fn set_locked(app: &AppHandle, locked: bool) {
	LOCKED.store(locked, Ordering::Relaxed);
	if let Some(window) = app.get_webview_window(OVERLAY) {
		let _ = window.set_ignore_cursor_events(locked);
	}
	let _ = app.emit("locked", locked);
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
		let Ok(updater) = app.updater() else {
			return;
		};
		let Ok(Some(update)) = updater.check().await else {
			return;
		};
		if let Ok(bytes) = update.download(|_, _| {}, || {}).await {
			pktmon::stop();
			let _ = update.install(bytes);
		}
	});
}

fn now() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_micros() as u64).unwrap_or_default()
}