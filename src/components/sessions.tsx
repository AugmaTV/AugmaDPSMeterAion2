import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronLeft, Lock, LockOpen, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BossBar } from "@/components/boss-bar";
import { Board } from "@/components/board";
import { clock, compact, date } from "@/lib/format";
import { npcName } from "@/lib/game-data";
import { totalRate, type Mode, type SessionSummary, type SessionView } from "@/lib/meter";

function title(session: SessionSummary) {
	return session.name ?? npcName(session.boss) ?? "Monde ouvert";
}

export function Sessions({ onBack }: { onBack: () => void }) {
	const [sessions, setSessions] = useState<SessionSummary[]>([]);
	const [opened, setOpened] = useState<SessionSummary | null>(null);
	const [renaming, setRenaming] = useState<{ id: number; name: string } | null>(null);
	const [deleting, setDeleting] = useState<number | null>(null);

	const refresh = () => invoke<SessionSummary[]>("list_sessions").then(setSessions);

	useEffect(() => {
		refresh();
	}, []);

	const rename = () => {
		if (renaming) {
			invoke("rename_session", renaming).then(refresh);
			setRenaming(null);
		}
	};

	if (opened) {
		return <SessionPage session={opened} onBack={() => setOpened(null)} />;
	}

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<Button variant="ghost" size="icon-sm" title="Retour au meter" onClick={onBack}>
					<ChevronLeft />
				</Button>
				<span className="font-semibold">Historique</span>
				<span className="ml-auto text-muted-foreground tabular-nums">{sessions.length} sessions</span>
			</header>
			<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
				{sessions.length === 0 && <p className="m-auto text-sm text-muted-foreground">Aucune session enregistrée</p>}
				{sessions.map((session) => (
					<div key={session.id} className="flex h-12 shrink-0 items-center gap-2 rounded-md bg-muted/40 px-2 text-sm">
						{renaming?.id === session.id ? (
							<input
								autoFocus
								value={renaming.name}
								placeholder={title(session)}
								onChange={(event) => setRenaming({ id: session.id, name: event.target.value })}
								onKeyDown={(event) => {
									if (event.key === "Enter") {
										rename();
									}
									if (event.key === "Escape") {
										setRenaming(null);
									}
								}}
								onBlur={() => setRenaming(null)}
								className="h-8 min-w-0 flex-1 rounded-md border bg-background px-2 outline-none"
							/>
						) : (
							<button type="button" onClick={() => setOpened(session)} className="flex min-w-0 flex-1 flex-col items-start text-left">
								<span className="w-full truncate font-medium">{title(session)}</span>
								<span className="text-xs text-muted-foreground tabular-nums">
									{date(session.start)} · {clock(session.duration)} · {session.fights} {session.fights > 1 ? "combats" : "combat"}
								</span>
							</button>
						)}
						<span className="w-16 text-right font-semibold tabular-nums">{compact(session.dps)}/s</span>
						<Button variant={session.locked ? "secondary" : "ghost"} size="icon-sm" title={session.locked ? "Session protégée : cliquer pour la déverrouiller" : "Protéger de la suppression automatique"} onClick={() => invoke("lock_session", { id: session.id, locked: !session.locked }).then(refresh)}>
							{session.locked ? <Lock /> : <LockOpen />}
						</Button>
						<Button variant="ghost" size="icon-sm" title="Renommer" onClick={() => setRenaming({ id: session.id, name: session.name ?? "" })}>
							<Pencil />
						</Button>
						<Button variant={deleting === session.id ? "destructive" : "ghost"} size="icon-sm" title={deleting === session.id ? "Cliquer à nouveau pour supprimer" : "Supprimer"} onMouseLeave={() => setDeleting(null)} onClick={() => (deleting === session.id ? invoke("delete_session", { id: session.id }).then(refresh) : setDeleting(session.id))}>
							<Trash2 />
						</Button>
					</div>
				))}
			</section>
		</main>
	);
}

function SessionPage({ session, onBack }: { session: SessionSummary; onBack: () => void }) {
	const [fight, setFight] = useState<number | null>(null);
	const [view, setView] = useState<SessionView | null>(null);
	const [mode, setMode] = useState<Mode>("damage");

	useEffect(() => {
		invoke<SessionView | null>("load_session", { id: session.id, fight }).then(setView);
	}, [session.id, fight]);

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<Button variant="ghost" size="icon-sm" title="Retour à l'historique" onClick={onBack}>
					<ChevronLeft />
				</Button>
				<span className="truncate font-semibold">{title(session)}</span>
				<span className="text-muted-foreground tabular-nums">{clock(view?.snapshot.duration ?? 0)}</span>
				<span className="ml-auto font-semibold tabular-nums lg:invisible">{compact(totalRate(view?.snapshot.players ?? [], mode))}/s</span>
			</header>
			<div className="flex shrink-0 gap-1 overflow-x-auto border-b px-2 py-1.5">
				<Button variant={fight === null ? "secondary" : "ghost"} size="xs" onClick={() => setFight(null)}>
					Session complète
				</Button>
				{view?.fights.map((entry, index) => (
					<Button key={index} variant={fight === index ? "secondary" : "ghost"} size="xs" onClick={() => setFight(index)}>
						{npcName(entry.boss) ?? `Combat ${index + 1}`} · {clock(entry.duration)}
					</Button>
				))}
			</div>
			{view?.snapshot.boss && <BossBar boss={view.snapshot.boss} />}
			<Board snapshot={view?.snapshot ?? null} mode={mode} onMode={setMode} />
		</main>
	);
}