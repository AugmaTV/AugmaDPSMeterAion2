import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Castle, ChartLine, History, Lock, LockOpen, PictureInPicture2, RotateCcw, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BossBar } from "@/components/boss-bar";
import { Board, Headline } from "@/components/board";
import { CopyButton } from "@/components/copy-button";
import { Sessions } from "@/components/sessions";
import { Timeline } from "@/components/timeline";
import { clock } from "@/lib/format";
import type { OverlayState, Snapshot, Tab } from "@/lib/meter";
import { summary } from "@/lib/summary";
import { cn } from "@/lib/utils";

export default function App() {
	const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
	const [page, setPage] = useState<"live" | "history">("live");
	const [mode, setMode] = useState<Tab>("damage");
	const [overlay, setOverlay] = useState<OverlayState>({ open: false, locked: false });
	const [partyOnly, setPartyOnly] = useState(true);
	const [dungeon, setDungeon] = useState(false);
	const [chart, setChart] = useState(false);

	useEffect(() => {
		const listeners = Promise.all([
			listen<Snapshot>("snapshot", (event) => setSnapshot(event.payload)),
			listen<OverlayState>("overlay", (event) => setOverlay(event.payload)),
		]);
		invoke<OverlayState>("overlay").then(setOverlay);
		invoke<boolean>("party_only").then(setPartyOnly);
		invoke<boolean>("dungeon").then(setDungeon);
		return () => {
			listeners.then((unlisteners) => unlisteners.forEach((unlisten) => unlisten()));
		};
	}, []);

	if (page === "history") {
		return <Sessions onBack={() => setPage("live")} />;
	}

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<span className={cn("size-2 rounded-full", snapshot?.status === "live" ? "bg-emerald-400" : "bg-muted-foreground")} />
				<span className="hidden font-semibold sm:inline">Augma DPS</span>
				<span className="text-muted-foreground tabular-nums">{clock(snapshot?.duration ?? 0)}</span>
				<span className="ml-auto font-semibold tabular-nums xl:invisible">
					<Headline players={snapshot?.players ?? []} tab={mode} />
				</span>
				<Button variant={partyOnly ? "secondary" : "ghost"} size="icon-sm" title={partyOnly ? "Afficher tous les joueurs" : "Afficher seulement moi et mon groupe"} onClick={() => invoke("set_party_only", { enabled: !partyOnly }).then(() => setPartyOnly(!partyOnly))}>
					<Users />
				</Button>
				<Button variant={dungeon ? "secondary" : "ghost"} size="icon-sm" title={dungeon ? "Afficher seulement le combat en cours" : "Cumuler tous les combats du donjon"} onClick={() => invoke("set_dungeon", { enabled: !dungeon }).then(() => setDungeon(!dungeon))}>
					<Castle />
				</Button>
				<Button variant={chart ? "secondary" : "ghost"} size="icon-sm" title={chart ? "Revenir aux listes" : "Courbe du combat"} onClick={() => setChart(!chart)}>
					<ChartLine />
				</Button>
				<CopyButton text={() => (snapshot ? summary(snapshot, null) : null)} />
				<Button variant="ghost" size="icon-sm" title="Historique des sessions" onClick={() => setPage("history")}>
					<History />
				</Button>
				{overlay.open && (
					<Button variant="ghost" size="icon-sm" title={overlay.locked ? "Déverrouiller les overlays (Ctrl+Shift+L)" : "Verrouiller les overlays (Ctrl+Shift+L)"} onClick={() => invoke("lock_overlay", { locked: !overlay.locked })}>
						{overlay.locked ? <Lock /> : <LockOpen />}
					</Button>
				)}
				{mode !== "deaths" && (
					<Button variant={overlay.open ? "secondary" : "ghost"} size="icon-sm" title="Ouvrir un overlay sur l'onglet actif" className="xl:hidden" onClick={() => invoke("open_overlay", { mode })}>
						<PictureInPicture2 />
					</Button>
				)}
				<Button variant="ghost" size="icon-sm" title="Réinitialiser" onClick={() => invoke("reset")}>
					<RotateCcw />
				</Button>
			</header>
			{snapshot?.boss && <BossBar boss={snapshot.boss} />}
			{chart ? <Timeline snapshot={snapshot} /> : <Board snapshot={snapshot} mode={mode} onMode={setMode} onOverlay={(column) => invoke("open_overlay", { mode: column })} />}
			<footer className="shrink-0 border-t px-3 py-1 text-[10px] text-muted-foreground">AION 2, noms et icônes © NCSOFT Corporation — projet non affilié à NCSOFT</footer>
		</main>
	);
}