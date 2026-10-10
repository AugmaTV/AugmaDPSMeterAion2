import { useSyncExternalStore } from "react";

const KEY = "trackedPets";
const OLD_KEY = "trackedPet";

const listeners = new Set<() => void>();

function stored() {
	try {
		const value = JSON.parse(localStorage.getItem(KEY) ?? "[]");
		if (Array.isArray(value)) {
			return value.filter((id): id is number => typeof id === "number");
		}
	} catch {
		return [];
	}
	return [];
}

function migrate() {
	const old = Number(localStorage.getItem(OLD_KEY));
	if (old > 0 && localStorage.getItem(KEY) === null) {
		localStorage.setItem(KEY, JSON.stringify([old]));
	}
	localStorage.removeItem(OLD_KEY);
}

migrate();

let current = stored();

window.addEventListener("storage", (event) => {
	if (event.key === KEY) {
		current = stored();
		listeners.forEach((listener) => listener());
	}
});

function update(next: number[]) {
	localStorage.setItem(KEY, JSON.stringify(next));
	current = next;
	listeners.forEach((listener) => listener());
}

export function toggleTrackedPet(id: number) {
	update(current.includes(id) ? current.filter((other) => other !== id) : [...current, id]);
}

export function addTrackedPet(id: number) {
	if (!current.includes(id)) {
		update([...current, id]);
	}
}

export function removeTrackedPet(id: number) {
	update(current.filter((other) => other !== id));
}

export function useTrackedPets() {
	return useSyncExternalStore(
		(listener) => {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
		() => current,
	);
}
