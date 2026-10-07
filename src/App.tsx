import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Lock, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type Status = "unavailable" | "waiting" | "live";

type Player = {
	id: number;
	name: string | null;
	class: number;
	damage: number;
	dps: number;
	hits: number;
	crits: number;
	own: boolean;
};

type Snapshot = {
	status: Status;
	duration: number;
	total: number;
	players: Player[];
};

const CLASSES = [
	{ name: "Inconnu", color: "bg-neutral-500" },
	{ name: "Gladiateur", color: "bg-amber-600" },
	{ name: "Templier", color: "bg-yellow-500" },
	{ name: "Assassin", color: "bg-fuchsia-600" },
	{ name: "Rôdeur", color: "bg-lime-600" },
	{ name: "Sorcier", color: "bg-sky-600" },
	{ name: "Élémentaliste", color: "bg-violet-600" },
	{ name: "Clerc", color: "bg-emerald-500" },
	{ name: "Aède", color: "bg-teal-600" },
	{ name: "Brawler", color: "bg-red-600" },
];

function format(value: number) {
	if (value >= 1_000_000) {
		return `${(value / 1_000_000).toFixed(2)}M`;
	}
	if (value >= 1_000) {
		return `${(value / 1_000).toFixed(1)}k`;
	}
	return `${value}`;
}

function clock(millis: number) {
	const seconds = Math.floor(millis / 1000);
	return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

export default function App() {
	const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
	const [locked, setLocked] = useState(true);

	useEffect(() => {
		const listeners = Promise.all([
			listen<Snapshot>("snapshot", (event) => setSnapshot(event.payload)),
			listen<boolean>("locked", (event) => setLocked(event.payload)),
		]);
		invoke<boolean>("locked").then(setLocked);
		return () => {
			listeners.then((unlisteners) => unlisteners.forEach((unlisten) => unlisten()));
		};
	}, []);

	const players = snapshot?.players ?? [];
	const top = players[0]?.damage || 1;
	const total = players.reduce((sum, player) => sum + player.dps, 0);

	return (
		<main className={cn("flex h-screen flex-col overflow-hidden rounded-lg border bg-neutral-950/75 text-neutral-100 select-none", locked ? "border-white/10" : "border-sky-400/70")}>
			<header data-tauri-drag-region className={cn("flex h-7 shrink-0 items-center gap-2 border-b border-white/10 px-2 text-xs", !locked && "cursor-move")}>
				<span data-tauri-drag-region className={cn("size-2 rounded-full", snapshot?.status === "live" ? "bg-emerald-400" : "bg-neutral-500")} />
				<span data-tauri-drag-region className="font-semibold">Augma DPS</span>
				<span data-tauri-drag-region className="text-white/60 tabular-nums">{clock(snapshot?.duration ?? 0)}</span>
				<span data-tauri-drag-region className="ml-auto tabular-nums">{format(total)}/s</span>
				{!locked && (
					<>
						<Button variant="ghost" size="icon-xs" title="Réinitialiser" onClick={() => invoke("reset")}>
							<RotateCcw />
						</Button>
						<Button variant="ghost" size="icon-xs" title="Verrouiller (Ctrl+Shift+L)" onClick={() => invoke("lock")}>
							<Lock />
						</Button>
					</>
				)}
			</header>
			<section className="flex flex-1 flex-col gap-1 overflow-y-auto p-1.5">
				{snapshot?.status === "unavailable" && (
					<p className="m-auto p-2 text-center text-xs">Impossible de lire le trafic réseau. Relance le meter et accepte la demande administrateur.</p>
				)}
				{(snapshot === null || snapshot.status === "waiting" || snapshot.status === "live") && players.length === 0 && (
					<p className="m-auto text-xs text-white/50">{snapshot?.status === "live" ? "En attente d'un combat…" : "En attente du jeu…"}</p>
				)}
				{players.map((player) => (
					<div key={player.id} title={`${CLASSES[player.class].name} · ${player.hits} coups · ${player.hits ? Math.round((player.crits / player.hits) * 100) : 0}% crit`} className="relative h-6 shrink-0 overflow-hidden rounded-sm bg-white/5">
						<div className={cn("absolute inset-y-0 left-0 opacity-60", CLASSES[player.class].color)} style={{ width: `${(player.damage / top) * 100}%` }} />
						<div className="relative flex h-full items-center gap-2 px-2 text-xs">
							<span className={cn("flex-1 truncate", player.own && "font-semibold")}>{player.name ?? `#${player.id}`}</span>
							<span className="tabular-nums">{format(player.dps)}</span>
							<span className="w-9 text-right text-white/60 tabular-nums">{Math.round((player.damage / (snapshot?.total || 1)) * 100)}%</span>
						</div>
					</div>
				))}
			</section>
			{!locked && <footer className="shrink-0 border-t border-white/10 px-2 py-1 text-[10px] text-white/50">Déplace la fenêtre par l'en-tête · Ctrl+Shift+L pour verrouiller</footer>}
		</main>
	);
}