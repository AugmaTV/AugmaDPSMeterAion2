mod engine;
pub mod frame;
mod meter;
pub mod net;
pub mod npcap;
pub mod packet;
pub mod pcap;
mod reader;
pub mod stream;

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{async_runtime, command, generate_context, generate_handler, AppHandle, Builder, Emitter, Manager, Result};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::StateFlags;

use crate::engine::Engine;
use crate::meter::{Meter, Status};
use crate::npcap::Npcap;

const OVERLAY: &str = "overlay";
const FILTER: &str = "tcp";
const LOCK_SHORTCUT: &str = "ctrl+shift+l";
const REFRESH: Duration = Duration::from_millis(500);
const RESCAN: Duration = Duration::from_secs(5);
const RETRY: Duration = Duration::from_secs(1);

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
		.plugin(tauri_plugin_opener::init())
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
		.run(generate_context!())
		.expect("error while running tauri application");
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
		let npcap = match Npcap::load() {
			Some(npcap) => npcap,
			None => {
				set_locked(&app, false);
				loop {
					idle(&app, Status::NpcapMissing);
					if let Some(npcap) = Npcap::load() {
						break npcap;
					}
				}
			}
		};
		let (sender, receiver) = mpsc::channel();
		let active = Arc::new(Mutex::new(HashSet::new()));
		let mut engine = Engine::default();
		let mut available = false;
		let mut scanned: Option<Instant> = None;
		let mut emitted = Instant::now();
		loop {
			if scanned.is_none_or(|instant| instant.elapsed() >= RESCAN) {
				available = npcap::listen(&npcap, FILTER, &sender, &active).is_ok_and(|count| count > 0);
				scanned = Some(Instant::now());
			}
			if let Ok(packet) = receiver.recv_timeout(REFRESH) {
				engine.process(packet.micros, packet.linktype, &packet.data);
			}
			if RESET.swap(false, Ordering::Relaxed) {
				engine.reset();
			}
			if emitted.elapsed() >= REFRESH {
				let snapshot = if available { engine.snapshot(now()) } else { Meter::default().snapshot(Status::NoDevice) };
				let _ = app.emit("snapshot", snapshot);
				emitted = Instant::now();
			}
		}
	});
}

fn idle(app: &AppHandle, status: Status) {
	let _ = app.emit("snapshot", Meter::default().snapshot(status));
	thread::sleep(RETRY);
}

fn update(app: AppHandle) {
	async_runtime::spawn(async move {
		let Ok(updater) = app.updater() else {
			return;
		};
		if let Ok(Some(update)) = updater.check().await {
			let _ = update.download_and_install(|_, _| {}, || {}).await;
		}
	});
}

fn now() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_micros() as u64).unwrap_or_default()
}