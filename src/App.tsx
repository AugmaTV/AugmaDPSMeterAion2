import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Castle, Lock, LockOpen, PictureInPicture2, RotateCcw, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { BossBar } from "@/components/boss-bar";
import { MeterView } from "@/components/meter-view";
import { clock, compact } from "@/lib/format";
import { MODES, TITLES, totalRate, type Mode, type OverlayState, type Snapshot } from "@/lib/meter";
import { cn } from "@/lib/utils";

export default function App() {
	const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
	const [selected, setSelected] = useState<{ id: number; mode: Mode } | null>(null);
	const [mode, setMode] = useState<Mode>("damage");
	const [overlay, setOverlay] = useState<OverlayState>({ open: false, locked: false });
	const [partyOnly, setPartyOnly] = useState(true);
	const [dungeon, setDungeon] = useState(false);

	useEffect(() => {
		const listeners = Promise.all([
			listen<Snapshot>("snapshot", (event) => {
				setSnapshot(event.payload);
				setSelected((selected) => (event.payload.players.some((player) => player.id === selected?.id) ? selected : null));
			}),
			listen<OverlayState>("overlay", (event) => setOverlay(event.payload)),
		]);
		invoke<OverlayState>("overlay").then(setOverlay);
		invoke<boolean>("party_only").then(setPartyOnly);
		invoke<boolean>("dungeon").then(setDungeon);
		return () => {
			listeners.then((unlisteners) => unlisteners.forEach((unlisten) => unlisten()));
		};
	}, []);

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<span className={cn("size-2 rounded-full", snapshot?.status === "live" ? "bg-emerald-400" : "bg-muted-foreground")} />
				<span className="font-semibold">Augma DPS</span>
				<span className="text-muted-foreground tabular-nums">{clock(snapshot?.duration ?? 0)}</span>
				<span className="ml-auto font-semibold tabular-nums lg:invisible">{compact(totalRate(snapshot?.players ?? [], mode))}/s</span>
				<Button variant={partyOnly ? "secondary" : "ghost"} size="icon-sm" title={partyOnly ? "Afficher tous les joueurs" : "Afficher seulement moi et mon groupe"} onClick={() => invoke("set_party_only", { enabled: !partyOnly }).then(() => setPartyOnly(!partyOnly))}>
					<Users />
				</Button>
				<Button variant={dungeon ? "secondary" : "ghost"} size="icon-sm" title={dungeon ? "Afficher seulement le combat en cours" : "Cumuler tous les combats du donjon"} onClick={() => invoke("set_dungeon", { enabled: !dungeon }).then(() => setDungeon(!dungeon))}>
					<Castle />
				</Button>
				{overlay.open && (
					<Button variant="ghost" size="icon-sm" title={overlay.locked ? "Déverrouiller les overlays (Ctrl+Shift+L)" : "Verrouiller les overlays (Ctrl+Shift+L)"} onClick={() => invoke("lock_overlay", { locked: !overlay.locked })}>
						{overlay.locked ? <Lock /> : <LockOpen />}
					</Button>
				)}
				<Button variant={overlay.open ? "secondary" : "ghost"} size="icon-sm" title="Ouvrir un overlay sur l'onglet actif" className="lg:hidden" onClick={() => invoke("open_overlay", { mode })}>
					<PictureInPicture2 />
				</Button>
				<Button variant="ghost" size="icon-sm" title="Réinitialiser" onClick={() => invoke("reset")}>
					<RotateCcw />
				</Button>
			</header>
			{snapshot?.boss && <BossBar boss={snapshot.boss} />}
			<Tabs value={mode} onValueChange={(value) => setMode(value as Mode)} className="shrink-0 px-2 pt-2 lg:hidden">
				<TabsList className="w-full">
					{MODES.map((tab) => (
						<TabsTrigger key={tab} value={tab}>
							{TITLES[tab]}
						</TabsTrigger>
					))}
				</TabsList>
			</Tabs>
			<div className="flex min-h-0 flex-1 flex-col lg:hidden">
				<MeterView snapshot={snapshot} mode={mode} selected={selected?.id ?? null} onSelect={(id) => setSelected(id === null ? null : { id, mode })} />
			</div>
			<div className="hidden min-h-0 flex-1 divide-x lg:flex">
				{MODES.map((column) => (
					<div key={column} className="flex min-w-0 flex-1 flex-col">
						<div className="flex h-9 shrink-0 items-center gap-2 border-b px-3 text-sm">
							<span className="font-semibold">{TITLES[column]}</span>
							<span className="ml-auto font-semibold tabular-nums">{compact(totalRate(snapshot?.players ?? [], column))}/s</span>
							<Button variant="ghost" size="icon-xs" title={`Ouvrir un overlay ${TITLES[column]}`} onClick={() => invoke("open_overlay", { mode: column })}>
								<PictureInPicture2 />
							</Button>
						</div>
						<MeterView snapshot={snapshot} mode={column} selected={selected?.mode === column ? selected.id : null} onSelect={(id) => setSelected(id === null ? null : { id, mode: column })} />
					</div>
				))}
			</div>
			<footer className="shrink-0 border-t px-3 py-1 text-[10px] text-muted-foreground">AION 2, noms et icônes © NCSOFT Corporation — projet non affilié à NCSOFT</footer>
		</main>
	);
}