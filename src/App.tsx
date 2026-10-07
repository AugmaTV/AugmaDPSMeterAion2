import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Lock, LockOpen, PictureInPicture2, RotateCcw, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { BossBar } from "@/components/boss-bar";
import { PlayerDetail } from "@/components/player-detail";
import { PlayerRow } from "@/components/player-row";
import { clock, compact } from "@/lib/format";
import { amount, rate, ranking, total, type Mode, type OverlayState, type Snapshot } from "@/lib/meter";
import { cn } from "@/lib/utils";

const EMPTY: Record<Mode, string> = {
	damage: "Aucun dégât",
	healing: "Aucun soin",
	taken: "Aucun dégât subi",
};

export default function App() {
	const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
	const [selected, setSelected] = useState<number | null>(null);
	const [mode, setMode] = useState<Mode>("damage");
	const [overlay, setOverlay] = useState<OverlayState>({ open: false, locked: false });
	const [partyOnly, setPartyOnly] = useState(true);

	useEffect(() => {
		const listeners = Promise.all([
			listen<Snapshot>("snapshot", (event) => {
				setSnapshot(event.payload);
				setSelected((selected) => (event.payload.players.some((player) => player.id === selected) ? selected : null));
			}),
			listen<OverlayState>("overlay", (event) => setOverlay(event.payload)),
		]);
		invoke<OverlayState>("overlay").then(setOverlay);
		invoke<boolean>("party_only").then(setPartyOnly);
		return () => {
			listeners.then((unlisteners) => unlisteners.forEach((unlisten) => unlisten()));
		};
	}, []);

	const players = ranking(snapshot?.players ?? [], mode);
	const player = snapshot?.players.find((player) => player.id === selected);
	const top = players[0] ? amount(players[0], mode) : 1;
	const totalRate = players.reduce((sum, player) => sum + rate(player, mode), 0);

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<span className={cn("size-2 rounded-full", snapshot?.status === "live" ? "bg-emerald-400" : "bg-muted-foreground")} />
				<span className="font-semibold">Augma DPS</span>
				<span className="text-muted-foreground tabular-nums">{clock(snapshot?.duration ?? 0)}</span>
				<span className="ml-auto font-semibold tabular-nums">{compact(totalRate)}/s</span>
				<Button variant={partyOnly ? "secondary" : "ghost"} size="icon-sm" title={partyOnly ? "Afficher tous les joueurs" : "Afficher seulement moi et mon groupe"} onClick={() => invoke("set_party_only", { enabled: !partyOnly }).then(() => setPartyOnly(!partyOnly))}>
					<Users />
				</Button>
				{overlay.open && (
					<Button variant="ghost" size="icon-sm" title={overlay.locked ? "Déverrouiller les overlays (Ctrl+Shift+L)" : "Verrouiller les overlays (Ctrl+Shift+L)"} onClick={() => invoke("lock_overlay", { locked: !overlay.locked })}>
						{overlay.locked ? <Lock /> : <LockOpen />}
					</Button>
				)}
				<Button variant={overlay.open ? "secondary" : "ghost"} size="icon-sm" title="Ouvrir un overlay sur l'onglet actif" onClick={() => invoke("open_overlay", { mode })}>
					<PictureInPicture2 />
				</Button>
				<Button variant="ghost" size="icon-sm" title="Réinitialiser" onClick={() => invoke("reset")}>
					<RotateCcw />
				</Button>
			</header>
			{snapshot?.boss && <BossBar boss={snapshot.boss} />}
			<Tabs value={mode} onValueChange={(value) => setMode(value as Mode)} className="shrink-0 px-2 pt-2">
				<TabsList className="w-full">
					<TabsTrigger value="damage">Dégâts</TabsTrigger>
					<TabsTrigger value="healing">Soins</TabsTrigger>
					<TabsTrigger value="taken">Subis</TabsTrigger>
				</TabsList>
			</Tabs>
			{player ? (
				<PlayerDetail player={player} mode={mode} onBack={() => setSelected(null)} />
			) : (
				<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
					{snapshot?.status === "unavailable" && <p className="m-auto p-4 text-center text-sm text-muted-foreground">Impossible de lire le trafic réseau. Relance le meter et accepte la demande administrateur.</p>}
					{snapshot?.status !== "unavailable" && players.length === 0 && <p className="m-auto text-sm text-muted-foreground">{snapshot?.players.length ? EMPTY[mode] : snapshot?.status === "live" ? "En attente d'un combat…" : "En attente du jeu…"}</p>}
					{players.map((player) => (
						<PlayerRow key={player.id} player={player} mode={mode} top={top} total={snapshot ? total(snapshot, mode) : 0} onSelect={() => setSelected(player.id)} />
					))}
				</section>
			)}
			<footer className="shrink-0 border-t px-3 py-1 text-[10px] text-muted-foreground">AION 2, noms et icônes © NCSOFT Corporation — projet non affilié à NCSOFT</footer>
		</main>
	);
}