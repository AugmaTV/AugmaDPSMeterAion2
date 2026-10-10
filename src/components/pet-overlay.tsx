import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Lock, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { PetIcon } from "@/components/pet-icon";
import { PET_MAX_LEVEL, PET_THRESHOLDS, petName } from "@/lib/game-data";
import { useDictionary } from "@/lib/i18n";
import type { OverlayState, Pet, Profile } from "@/lib/meter";
import { addTrackedPet, removeTrackedPet, useTrackedPets } from "@/lib/tracking";
import { cn } from "@/lib/utils";

function Row({ pet, removable }: { pet: Pet; removable: boolean }) {
	const maxed = pet.level >= PET_MAX_LEVEL;
	const threshold = PET_THRESHOLDS[pet.level] ?? 0;
	return (
		<div className="relative flex h-9 shrink-0 items-center gap-2 overflow-hidden rounded-sm bg-white/5 px-1.5 text-xs">
			<span className="absolute inset-y-0 left-0 bg-sky-400/25" style={{ width: maxed ? "100%" : `${Math.min(100, (pet.progress / threshold) * 100)}%` }} />
			<PetIcon id={pet.id} className="relative size-6" />
			<span className="relative min-w-0 flex-1 truncate font-semibold">{petName(pet.id)}</span>
			<span className="relative shrink-0 text-white/60 tabular-nums">
				{pet.level}/{PET_MAX_LEVEL}
			</span>
			<span className="relative w-10 shrink-0 text-right font-semibold text-sky-300 tabular-nums">{maxed ? "MAX" : `${pet.progress}/${threshold}`}</span>
			{removable && (
				<button type="button" onClick={() => removeTrackedPet(pet.id)} className="relative shrink-0 text-white/40 hover:text-white">
					<X className="size-3" />
				</button>
			)}
		</div>
	);
}

export function PetOverlay() {
	const t = useDictionary();
	const [profile, setProfile] = useState<Profile | null>(null);
	const [locked, setLocked] = useState(false);
	const tracked = useTrackedPets();

	useEffect(() => {
		const listeners = Promise.all([listen<Profile | null>("profile", (event) => setProfile(event.payload)), listen<OverlayState>("overlay", (event) => setLocked(event.payload.locked))]);
		invoke<Profile | null>("profile").then(setProfile);
		invoke<OverlayState>("overlay").then((state) => setLocked(state.locked));
		return () => {
			listeners.then((unlisteners) => unlisteners.forEach((unlisten) => unlisten()));
		};
	}, []);

	const pets = profile?.pets ?? [];
	const ids = tracked.length > 0 ? tracked : profile?.lastPet != null ? [profile.lastPet] : [];
	const rows = ids.map((id) => pets.find((pet) => pet.id === id) ?? { id, level: 0, progress: 0 });
	const available = pets.filter((pet) => !tracked.includes(pet.id)).sort((a, b) => petName(a.id).localeCompare(petName(b.id)));

	return (
		<main className={cn("flex h-screen flex-col overflow-hidden rounded-lg border bg-neutral-950/75 text-neutral-100 select-none", locked ? "border-white/10" : "border-sky-400/70")}>
			<header data-tauri-drag-region className="flex h-7 shrink-0 items-center gap-2 border-b border-white/10 px-2 text-xs">
				<span data-tauri-drag-region className="font-semibold">{t.character.pets}</span>
				{!locked && (
					<select value="" onChange={(event) => event.target.value && addTrackedPet(Number(event.target.value))} title={t.character.track} className="ml-auto h-5 min-w-0 flex-1 rounded border border-white/15 bg-neutral-900 px-1 text-[10px] text-neutral-100">
						<option value="">+ {t.character.addPet}</option>
						{available.map((pet) => (
							<option key={pet.id} value={pet.id}>
								{petName(pet.id)}
							</option>
						))}
					</select>
				)}
				{!locked && (
					<span className="flex">
						<Button variant="ghost" size="icon-xs" title={t.overlay.lock} onClick={() => invoke("lock_overlay", { locked: true })}>
							<Lock />
						</Button>
						<Button variant="ghost" size="icon-xs" title={t.overlay.close} onClick={() => getCurrentWindow().close()}>
							<X />
						</Button>
					</span>
				)}
			</header>
			<section data-tauri-drag-region className="flex flex-1 flex-col gap-1 overflow-hidden p-1.5">
				{rows.length === 0 ? <span className="m-auto text-xs text-white/60">{t.character.noPet}</span> : rows.map((pet) => <Row key={pet.id} pet={pet} removable={!locked && tracked.includes(pet.id)} />)}
			</section>
		</main>
	);
}
