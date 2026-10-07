import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { BossBar } from "@/components/boss-bar";
import { ClassIcon } from "@/components/class-icon";
import { CLASSES } from "@/lib/classes";
import { clock, compact, percent } from "@/lib/format";
import { amount, rate, ranking, total, type Mode, type OverlayState, type Snapshot } from "@/lib/meter";
import { cn } from "@/lib/utils";

export function Overlay() {
	const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
	const [locked, setLocked] = useState(false);
	const [mode, setMode] = useState<Mode>("damage");

	useEffect(() => {
		const listeners = Promise.all([
			listen<Snapshot>("snapshot", (event) => setSnapshot(event.payload)),
			listen<OverlayState>("overlay", (event) => setLocked(event.payload.locked)),
		]);
		invoke<OverlayState>("overlay").then((state) => setLocked(state.locked));
		return () => {
			listeners.then((unlisteners) => unlisteners.forEach((unlisten) => unlisten()));
		};
	}, []);

	const players = ranking(snapshot?.players ?? [], mode);
	const top = players[0] ? amount(players[0], mode) : 1;
	const totalRate = players.reduce((sum, player) => sum + rate(player, mode), 0);

	return (
		<main className={cn("flex h-screen flex-col overflow-hidden rounded-lg border bg-neutral-950/75 text-neutral-100 select-none", locked ? "border-white/10" : "border-sky-400/70")}>
			<header data-tauri-drag-region className="flex h-7 shrink-0 items-center gap-2 border-b border-white/10 px-2 text-xs">
				<span data-tauri-drag-region className={cn("size-2 rounded-full", snapshot?.status === "live" ? "bg-emerald-400" : "bg-neutral-500")} />
				<span data-tauri-drag-region className="text-white/60 tabular-nums">{clock(snapshot?.duration ?? 0)}</span>
				<span data-tauri-drag-region className="ml-auto font-semibold tabular-nums">{compact(totalRate)}/s</span>
				{!locked && (
					<Button variant="ghost" size="icon-xs" title="Verrouiller (Ctrl+Shift+L)" onClick={() => invoke("lock_overlay", { locked: true })}>
						<Lock />
					</Button>
				)}
			</header>
			{snapshot?.boss && <BossBar boss={snapshot.boss} compactView />}
			<Tabs value={mode} onValueChange={(value) => setMode(value as Mode)} className="shrink-0 px-1.5 pt-1.5">
				<TabsList className="w-full bg-white/5 group-data-horizontal/tabs:h-6">
					<TabsTrigger value="damage" className="text-[11px]">Dégâts</TabsTrigger>
					<TabsTrigger value="healing" className="text-[11px]">Soins</TabsTrigger>
					<TabsTrigger value="taken" className="text-[11px]">Subis</TabsTrigger>
				</TabsList>
			</Tabs>
			<section className="flex flex-1 flex-col gap-1 overflow-hidden p-1.5">
				{players.map((player) => (
					<div key={player.id} className="relative flex h-6 shrink-0 items-center gap-1.5 overflow-hidden rounded-sm bg-white/5 px-1.5 text-xs">
						<span className="absolute inset-y-0 left-0 opacity-40" style={{ width: `${(amount(player, mode) / top) * 100}%`, backgroundColor: CLASSES[player.class].color }} />
						<ClassIcon gameClass={player.class} className="relative size-4" />
						<span className={cn("relative flex-1 truncate", player.own && "font-semibold")}>{player.name ?? CLASSES[player.class].name}</span>
						<span className="relative tabular-nums">{compact(rate(player, mode))}</span>
						<span className="relative w-8 text-right text-white/60 tabular-nums">{percent(amount(player, mode), snapshot ? total(snapshot, mode) : 0)}</span>
					</div>
				))}
			</section>
		</main>
	);
}