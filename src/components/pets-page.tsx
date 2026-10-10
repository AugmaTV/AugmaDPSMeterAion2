import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { PictureInPicture2, Pin } from "lucide-react";
import { Perception } from "@/components/character";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { PetIcon } from "@/components/pet-icon";
import { knownPets, PET_MAX_LEVEL, PET_THRESHOLDS, petName } from "@/lib/game-data";
import { useDictionary } from "@/lib/i18n";
import type { Pet, Profile } from "@/lib/meter";
import { toggleTrackedPet, useTrackedPets } from "@/lib/tracking";
import { cn } from "@/lib/utils";

function Card({ pet, tracked, last }: { pet: Pet; tracked: boolean; last: boolean }) {
	const t = useDictionary();
	const maxed = pet.level >= PET_MAX_LEVEL;
	const threshold = PET_THRESHOLDS[pet.level] ?? 0;
	return (
		<button type="button" title={t.character.track} onClick={() => toggleTrackedPet(pet.id)} className={cn("relative flex flex-col gap-2 rounded-lg border p-2 text-left transition-colors", tracked ? "border-sky-400 bg-sky-500/15" : last ? "border-violet-400/60 bg-violet-500/10" : "bg-muted/30 hover:bg-muted/60", pet.level === 0 && "opacity-60")}>
			{tracked && <Pin className="absolute top-1.5 right-1.5 size-3.5 text-sky-400" />}
			<span className="flex items-center gap-2">
				<PetIcon id={pet.id} className="size-10" />
				<span className="flex min-w-0 flex-1 flex-col">
					<span className="truncate text-sm font-semibold">{petName(pet.id)}</span>
					<span className="flex gap-0.5" title={`${pet.level}/${PET_MAX_LEVEL}`}>
						{Array.from({ length: PET_MAX_LEVEL }, (_, index) => (
							<span key={index} className={cn("h-1.5 w-4 rounded-full", index < pet.level ? "bg-violet-400" : "bg-white/15")} />
						))}
					</span>
				</span>
			</span>
			<span className="flex flex-col gap-1">
				<span className="h-1.5 w-full overflow-hidden rounded-full bg-sky-400/15">
					<span className="block h-full rounded-full bg-sky-400" style={{ width: maxed ? "100%" : `${Math.min(100, (pet.progress / threshold) * 100)}%` }} />
				</span>
				<span className="flex justify-between text-[10px] text-muted-foreground tabular-nums">
					<span>{t.detail.level(pet.level)}</span>
					<span>{maxed ? "MAX" : `${pet.progress}/${threshold}`}</span>
				</span>
			</span>
		</button>
	);
}

export function Pets() {
	const t = useDictionary();
	const tracked = useTrackedPets();
	const [profile, setProfile] = useState<Profile | null>(null);

	useEffect(() => {
		const listener = listen<Profile | null>("profile", (event) => setProfile(event.payload));
		invoke<Profile | null>("profile").then(setProfile);
		return () => {
			listener.then((unlisten) => unlisten());
		};
	}, []);

	const received = new Set((profile?.pets ?? []).map((pet) => pet.id));
	const missing = knownPets().filter((id) => !received.has(id)).map((id): Pet => ({ id, level: 0, progress: 0 }));
	const pets = [...(profile?.pets ?? []), ...missing].sort((a, b) => b.level - a.level || b.progress - a.progress || petName(a.id).localeCompare(petName(b.id)));
	const owned = pets.filter((pet) => pet.level > 0).length;

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<span className="font-semibold">{t.character.pets}</span>
				<span className="text-muted-foreground tabular-nums">
					{owned}/{pets.length}
				</span>
				<Button variant="ghost" size="icon-sm" className="ml-auto" title={t.character.petOverlay} onClick={() => invoke("open_overlay", { mode: "pets" })}>
					<PictureInPicture2 />
				</Button>
			</header>
			<Tabs defaultValue="pets" className="flex min-h-0 flex-1 flex-col gap-0">
				<TabsList className="mx-3 mt-2 w-auto shrink-0 self-start">
					<TabsTrigger value="pets">{t.character.pets}</TabsTrigger>
					<TabsTrigger value="perception">{t.character.perception}</TabsTrigger>
				</TabsList>
				<TabsContent value="pets" className="min-h-0 flex-1 overflow-y-auto">
					{pets.length === 0 ? (
						<p className="m-auto max-w-sm p-4 text-center text-sm text-muted-foreground">{t.character.empty}</p>
					) : (
						<section className="grid grid-cols-2 content-start gap-2 p-3 md:grid-cols-3 xl:grid-cols-4">
							{pets.map((pet) => (
								<Card key={pet.id} pet={pet} tracked={tracked.includes(pet.id)} last={profile?.lastPet === pet.id} />
							))}
						</section>
					)}
				</TabsContent>
				<TabsContent value="perception" className="min-h-0 flex-1 overflow-y-auto p-3 text-xs">
					<Perception species={profile?.perception ?? []} />
				</TabsContent>
			</Tabs>
		</div>
	);
}
