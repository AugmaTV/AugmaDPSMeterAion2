import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Castle, ChartLine, History, LayoutDashboard, Lock, LockOpen, PawPrint, PictureInPicture2, RotateCcw, Shirt, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BossBar } from "@/components/boss-bar";
import { Board, Headline } from "@/components/board";
import { Character } from "@/components/character";
import { CopyButton } from "@/components/copy-button";
import { Pets } from "@/components/pets-page";
import { Sessions } from "@/components/sessions";
import { Timeline } from "@/components/timeline";
import { clock } from "@/lib/format";
import { language, LANGUAGES, setLanguage, useDictionary } from "@/lib/i18n";
import type { OverlayState, Snapshot, Tab } from "@/lib/meter";
import { summary } from "@/lib/summary";
import { cn } from "@/lib/utils";

export default function App() {
	const t = useDictionary();
	const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
	const [page, setPage] = useState<"live" | "history" | "character" | "pets">("live");
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

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<span className={cn("size-2 rounded-full", snapshot?.status === "live" ? "bg-emerald-400" : "bg-muted-foreground")} />
				<span className="hidden font-semibold sm:inline">Augma DPS</span>
				<span className="text-muted-foreground tabular-nums">{clock(snapshot?.duration ?? 0)}</span>
				<span className="ml-auto font-semibold tabular-nums xl:invisible">
					<Headline players={snapshot?.players ?? []} tab={mode} />
				</span>
				<span className="h-5 w-px shrink-0 bg-border" />
				<Button variant={page === "live" ? "secondary" : "ghost"} size="icon-sm" title={t.header.meter} onClick={() => setPage("live")}>
					<LayoutDashboard />
				</Button>
				<Button variant={page === "character" ? "secondary" : "ghost"} size="icon-sm" title={t.header.character} onClick={() => setPage("character")}>
					<Shirt />
				</Button>
				<Button variant={page === "pets" ? "secondary" : "ghost"} size="icon-sm" title={t.header.pets} onClick={() => setPage("pets")}>
					<PawPrint />
				</Button>
				<span className="h-5 w-px shrink-0 bg-border" />
				<Button variant={partyOnly ? "secondary" : "ghost"} size="icon-sm" title={partyOnly ? t.header.allPlayers : t.header.partyOnly} onClick={() => invoke("set_party_only", { enabled: !partyOnly }).then(() => setPartyOnly(!partyOnly))}>
					<Users />
				</Button>
				<Button variant={dungeon ? "secondary" : "ghost"} size="icon-sm" title={dungeon ? t.header.currentFight : t.header.dungeon} onClick={() => invoke("set_dungeon", { enabled: !dungeon }).then(() => setDungeon(!dungeon))}>
					<Castle />
				</Button>
				<Button variant={chart && page === "live" ? "secondary" : "ghost"} size="icon-sm" title={chart && page === "live" ? t.header.lists : t.header.chart} onClick={() => { setChart(page === "live" ? !chart : true); setPage("live"); }}>
					<ChartLine />
				</Button>
				<CopyButton text={() => (snapshot ? summary(snapshot, null) : null)} />
				{overlay.open && (
					<Button variant="ghost" size="icon-sm" title={overlay.locked ? t.header.unlockOverlays : t.header.lockOverlays} onClick={() => invoke("lock_overlay", { locked: !overlay.locked })}>
						{overlay.locked ? <Lock /> : <LockOpen />}
					</Button>
				)}
				{mode !== "deaths" && (
					<Button variant={overlay.open ? "secondary" : "ghost"} size="icon-sm" title={t.header.openOverlay} className="xl:hidden" onClick={() => invoke("open_overlay", { mode })}>
						<PictureInPicture2 />
					</Button>
				)}
				<span className="h-5 w-px shrink-0 bg-border" />
				<Button variant={page === "history" ? "secondary" : "ghost"} size="icon-sm" title={t.header.history} onClick={() => setPage("history")}>
					<History />
				</Button>
				<Button variant="ghost" size="icon-sm" title={t.header.reset} onClick={() => invoke("reset")}>
					<RotateCcw />
				</Button>
			</header>
			{page === "history" && <Sessions />}
			{page === "character" && <Character />}
			{page === "pets" && <Pets />}
			{page === "live" && snapshot?.boss && <BossBar boss={snapshot.boss} />}
			{page === "live" && (chart ? <Timeline snapshot={snapshot} /> : <Board snapshot={snapshot} mode={mode} onMode={setMode} onOverlay={(column) => invoke("open_overlay", { mode: column })} />)}
			<footer className="flex shrink-0 items-center gap-2 border-t px-3 py-1 text-[10px] text-muted-foreground">
				<span className="truncate">{t.footer}</span>
				<span className="ml-auto flex shrink-0 gap-0.5" title={t.language}>
					{LANGUAGES.map((option) => (
						<button key={option} type="button" onClick={() => setLanguage(option)} className={cn("rounded px-1.5 font-semibold uppercase transition-colors", option === language() ? "bg-secondary text-foreground" : "hover:text-foreground")}>
							{option}
						</button>
					))}
				</span>
			</footer>
		</main>
	);
}