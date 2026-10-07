use std::env;

use tauri_build::{Attributes, WindowsAttributes};

fn main() {
	let mut attributes = Attributes::new();
	if env::var("PROFILE").as_deref() == Ok("release") {
		attributes = attributes.windows_attributes(WindowsAttributes::new().app_manifest(include_str!("manifest.xml")));
	}
	tauri_build::try_build(attributes).expect("failed to run build script");
}