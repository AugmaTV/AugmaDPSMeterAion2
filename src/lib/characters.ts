import { useSyncExternalStore } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Profile } from "@/lib/meter";

type State = { characters: Profile[]; current: string | null; chosen: string | null; selected: Profile | null };

const listeners = new Set<() => void>();

let characters: Profile[] = [];
let current: string | null = null;
let chosen: string | null = null;
let state = build();

function build(): State {
	const selected = characters.find((profile) => profile.name === chosen) ?? characters.find((profile) => profile.name === current) ?? null;
	return { characters, current, chosen: selected?.name ?? null, selected };
}

function publish() {
	state = build();
	listeners.forEach((listener) => listener());
}

function refresh() {
	invoke<Profile[]>("characters").then((list) => {
		characters = list;
		publish();
	});
}

listen<Profile | null>("profile", (event) => {
	const name = event.payload?.name ?? null;
	if (name !== current) {
		chosen = null;
	}
	current = name;
	refresh();
});

invoke<Profile | null>("profile").then((profile) => {
	current = profile?.name ?? null;
	refresh();
});

export function selectCharacter(name: string) {
	chosen = name;
	publish();
}

export function useCharacters() {
	return useSyncExternalStore(
		(listener) => {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
		() => state,
	);
}
