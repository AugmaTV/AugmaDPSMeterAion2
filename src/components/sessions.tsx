import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChartLine, ChevronLeft, Lock, LockOpen, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { BossBar } from "@/components/boss-bar";
import { Board, Headline } from "@/components/board";
import { CopyButton } from "@/components/copy-button";
import { Timeline } from "@/components/timeline";
import { clock, compact, date } from "@/lib/format";
import { npcName } from "@/lib/game-data";
import { useDictionary, type Dictionary } from "@/lib/i18n";
import { totalRate, type Player, type SessionSummary, type SessionView, type Tab } from "@/lib/meter";
import { summary } from "@/lib/summary";

function title(session: SessionSummary, t: Dictionary) {
	return session.name ?? npcName(session.boss) ?? t.sessions.openWorld;
}

export function Sessions({ onBack }: { onBack: () => void }) {
	const t = useDictionary();
	const [sessions, setSessions] = useState<SessionSummary[]>([]);
	const [opened, setOpened] = useState<SessionSummary | null>(null);
	const [renaming, setRenaming] = useState<{ id: number; name: string } | null>(null);
	const [deleting, setDeleting] = useState<number | null>(null);
	const [selected, setSelected] = useState<number[]>([]);
	const [comparing, setComparing] = useState<SessionSummary[] | null>(null);

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

	const toggle = (id: number) => setSelected(selected.includes(id) ? selected.filter((other) => other !== id) : [...selected, id]);

	if (opened) {
		return <SessionPage session={opened} onBack={() => setOpened(null)} />;
	}

	if (comparing) {
		return <SessionCompare sessions={comparing} onBack={() => setComparing(null)} />;
	}

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<Button variant="ghost" size="icon-sm" title={t.sessions.backToMeter} onClick={onBack}>
					<ChevronLeft />
				</Button>
				<span className="font-semibold">{t.sessions.title}</span>
				<span className="ml-auto text-muted-foreground tabular-nums">{t.sessions.count(sessions.length)}</span>
			</header>
			<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
				{sessions.length === 0 && <p className="m-auto text-sm text-muted-foreground">{t.sessions.empty}</p>}
				{sessions.length > 0 && (
					<div className="flex h-9 shrink-0 items-center gap-2 px-2 text-xs text-muted-foreground">
						<input type="checkbox" checked={selected.length === sessions.length} onChange={() => setSelected(selected.length === sessions.length ? [] : sessions.map((session) => session.id))} className="size-4 accent-sky-400" />
						<span>{t.sessions.selectAll}</span>
						<span className="tabular-nums">· {t.sessions.selected(selected.length)}</span>
						<Button variant="secondary" size="xs" className="ml-auto" disabled={selected.length !== 2} title={selected.length === 2 ? t.sessions.compareTitle : t.sessions.compareHint} onClick={() => setComparing(sessions.filter((session) => selected.includes(session.id)))}>
							{t.sessions.compare}
						</Button>
					</div>
				)}
				{sessions.map((session) => (
					<div key={session.id} className="flex h-12 shrink-0 items-center gap-2 rounded-md bg-muted/40 px-2 text-sm">
						<input type="checkbox" checked={selected.includes(session.id)} onChange={() => toggle(session.id)} className="size-4 shrink-0 accent-sky-400" />
						{renaming?.id === session.id ? (
							<input
								autoFocus
								value={renaming.name}
								placeholder={title(session, t)}
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
								<span className="w-full truncate font-medium">{title(session, t)}</span>
								<span className="text-xs text-muted-foreground tabular-nums">
									{date(session.start)} · {clock(session.duration)} · {t.sessions.fights(session.fights)}
								</span>
							</button>
						)}
						<span className="w-16 text-right font-semibold tabular-nums">{compact(session.dps)}/s</span>
						<Button variant={session.locked ? "secondary" : "ghost"} size="icon-sm" title={session.locked ? t.sessions.locked : t.sessions.lock} onClick={() => invoke("lock_session", { id: session.id, locked: !session.locked }).then(refresh)}>
							{session.locked ? <Lock /> : <LockOpen />}
						</Button>
						<Button variant="ghost" size="icon-sm" title={t.sessions.rename} onClick={() => setRenaming({ id: session.id, name: session.name ?? "" })}>
							<Pencil />
						</Button>
						<Button variant={deleting === session.id ? "destructive" : "ghost"} size="icon-sm" title={deleting === session.id ? t.sessions.confirmDelete : t.sessions.delete} onMouseLeave={() => setDeleting(null)} onClick={() => (deleting === session.id ? invoke("delete_session", { id: session.id }).then(refresh) : setDeleting(session.id))}>
							<Trash2 />
						</Button>
					</div>
				))}
			</section>
		</main>
	);
}

function SessionPage({ session, onBack }: { session: SessionSummary; onBack: () => void }) {
	const t = useDictionary();
	const [fight, setFight] = useState<number | null>(null);
	const [view, setView] = useState<SessionView | null>(null);
	const [mode, setMode] = useState<Tab>("damage");
	const [chart, setChart] = useState(false);

	useEffect(() => {
		invoke<SessionView | null>("load_session", { id: session.id, fight }).then(setView);
	}, [session.id, fight]);

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<Button variant="ghost" size="icon-sm" title={t.sessions.backToHistory} onClick={onBack}>
					<ChevronLeft />
				</Button>
				<span className="truncate font-semibold">{title(session, t)}</span>
				<span className="text-muted-foreground tabular-nums">{clock(view?.snapshot.duration ?? 0)}</span>
				<span className="ml-auto font-semibold tabular-nums xl:invisible">
					<Headline players={view?.snapshot.players ?? []} tab={mode} />
				</span>
				<Button variant={chart ? "secondary" : "ghost"} size="icon-sm" title={chart ? t.header.lists : t.header.chart} onClick={() => setChart(!chart)}>
					<ChartLine />
				</Button>
				<CopyButton text={() => (view ? summary(view.snapshot, fight === null ? title(session, t) : (npcName(view.fights[fight]?.boss ?? null) ?? t.sessions.fight(fight + 1))) : null)} />
			</header>
			<div className="flex shrink-0 gap-1 overflow-x-auto border-b px-2 py-1.5">
				<Button variant={fight === null ? "secondary" : "ghost"} size="xs" onClick={() => setFight(null)}>
					{t.sessions.whole}
				</Button>
				{view?.fights.map((entry, index) => (
					<Button key={index} variant={fight === index ? "secondary" : "ghost"} size="xs" onClick={() => setFight(index)}>
						{npcName(entry.boss) ?? t.sessions.fight(index + 1)} · {clock(entry.duration)}
					</Button>
				))}
			</div>
			{view?.snapshot.boss && <BossBar boss={view.snapshot.boss} />}
			{chart ? <Timeline snapshot={view?.snapshot ?? null} /> : <Board snapshot={view?.snapshot ?? null} mode={mode} onMode={setMode} />}
		</main>
	);
}

function identity(player: Player, t: Dictionary) {
	return player.name ?? `${t.classes[player.class]} #${player.id}`;
}

function SessionCompare({ sessions, onBack }: { sessions: SessionSummary[]; onBack: () => void }) {
	const t = useDictionary();
	const [views, setViews] = useState<(SessionView | null)[]>([]);

	useEffect(() => {
		Promise.all(sessions.map((session) => invoke<SessionView | null>("load_session", { id: session.id, fight: null }))).then(setViews);
	}, [sessions]);

	const find = (index: number, name: string) => views[index]?.snapshot.players.find((player) => identity(player, t) === name);
	const dps = (index: number, name: string) => find(index, name)?.dps ?? 0;
	const names = [...new Set(views.flatMap((view) => (view?.snapshot.players ?? []).map((player) => identity(player, t))))].sort((a, b) => Math.max(dps(0, b), dps(1, b)) - Math.max(dps(0, a), dps(1, a)));

	return (
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<Button variant="ghost" size="icon-sm" title={t.sessions.backToHistory} onClick={onBack}>
					<ChevronLeft />
				</Button>
				<span className="font-semibold">{t.sessions.comparison}</span>
			</header>
			<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-3 text-sm">
				<div className="grid grid-cols-[1fr_6rem_6rem_4rem] gap-x-2 gap-y-1 tabular-nums">
					<span />
					{sessions.map((session) => (
						<span key={session.id} className="truncate text-right font-semibold">
							{title(session, t)}
						</span>
					))}
					<span className="text-right text-muted-foreground">{t.sessions.gap}</span>
					<span className="text-muted-foreground">{t.sessions.date}</span>
					{sessions.map((session) => (
						<span key={session.id} className="text-right text-xs text-muted-foreground">
							{date(session.start)}
						</span>
					))}
					<span />
					<span className="text-muted-foreground">{t.sessions.duration}</span>
					{sessions.map((session) => (
						<span key={session.id} className="text-right">
							{clock(session.duration)}
						</span>
					))}
					<span />
					<span className="text-muted-foreground">{t.sessions.groupDps}</span>
					{views.map((view, index) => (
						<span key={index} className="text-right font-semibold">
							{compact(totalRate(view?.snapshot.players ?? [], "damage"))}/s
						</span>
					))}
					<span />
				</div>
				<div className="mt-2 grid grid-cols-[1fr_6rem_6rem_4rem] gap-x-2 gap-y-1 border-t pt-2 tabular-nums">
					{names.map((name) => {
						const before = dps(0, name);
						const after = dps(1, name);
						return (
							<span key={name} className="contents">
								<span className="truncate">{name}</span>
								<span className="text-right">{before ? `${compact(before)}/s` : "—"}</span>
								<span className="text-right">{after ? `${compact(after)}/s` : "—"}</span>
								<span className="text-right text-muted-foreground">{before && after ? `${after >= before ? "+" : ""}${Math.round(((after - before) / before) * 100)} %` : "—"}</span>
							</span>
						);
					})}
				</div>
			</section>
		</main>
	);
}