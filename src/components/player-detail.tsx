import { useState } from "react";
import { ChevronDown, ChevronLeft, Skull } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ClassIcon } from "@/components/class-icon";
import { SkillIcon } from "@/components/skill-icon";
import { CLASSES } from "@/lib/classes";
import { compact, percent } from "@/lib/format";
import { npcName, skillName, specializations } from "@/lib/game-data";
import { amount, rate, type Mode, type Player, type Skill, type Uptime } from "@/lib/meter";
import { cn } from "@/lib/utils";

const ENTRIES: Record<Mode, (player: Player) => Skill[]> = {
	damage: (player) => player.skills,
	healing: (player) => player.heals,
	taken: (player) => player.sources,
};

const STATS: Record<Mode, (player: Player, duration: number) => [string, string][]> = {
	damage: (player, duration) => [
		["Total", compact(player.damage)],
		["DPS actif", `${compact(player.active ? (player.damage * 1000) / player.active : 0)}/s`],
		["Actif", percent(player.active, duration)],
		["Critique", percent(player.crits, player.hits)],
		["Perfect", percent(player.perfect, player.hits)],
		["Puissant", percent(player.hard, player.hits)],
		["Dos", percent(player.back, player.hits)],
		["Face", percent(player.front, player.hits)],
		["Additionnels", percent(player.additional, player.hits)],
	],
	healing: (player) => [
		["Total", compact(player.healing)],
		["Soins", String(player.heals.reduce((sum, heal) => sum + heal.hits, 0))],
		["Utiles", percent(player.effective, player.healing)],
		["Surplus", compact(player.healing - player.effective)],
	],
	taken: (player) => [
		["Net", compact(player.taken)],
		["Brut", compact(player.taken + player.absorbed)],
		["Absorbé", percent(player.absorbed, player.taken + player.absorbed)],
		["Coups reçus", String(player.takenHits)],
		["Esquives", String(player.evasions)],
		["Résistances", String(player.resists)],
		["Parades", String(player.parries)],
		["Blocages", String(player.blocks)],
		["Perfect Block", String(player.perfectBlocks)],
		["Mur de fer", String(player.ironWalls)],
	],
};

function label(mode: Mode, entry: Skill) {
	if (mode === "taken") {
		return npcName(entry.id) ?? (entry.id ? `#${entry.id}` : "Inconnu");
	}
	return skillName(entry.id);
}

function details(mode: Mode, entry: Skill) {
	if (mode === "healing") {
		return `${entry.hits}×`;
	}
	if (!entry.hits) {
		return "DoT";
	}
	return mode === "damage" ? `${entry.casts} lanc.` : `${entry.hits} coups`;
}

function Tile({ title, value }: { title: string; value: string }) {
	return (
		<span className="flex flex-col rounded-md bg-background/60 px-2 py-1">
			<span className="text-[10px] text-muted-foreground">{title}</span>
			<span className="font-semibold">{value}</span>
		</span>
	);
}

function Meter({ title, part, total }: { title: string; part: number; total: number }) {
	return (
		<span className="flex items-center gap-2">
			<span className="w-16 shrink-0 text-muted-foreground">{title}</span>
			<span className="h-1.5 flex-1 overflow-hidden rounded-full bg-sky-400/15">
				<span className="block h-full rounded-full bg-sky-400" style={{ width: `${total ? Math.min((part / total) * 100, 100) : 0}%` }} />
			</span>
			<span className="w-10 shrink-0 text-right font-semibold tabular-nums">{percent(part, total)}</span>
		</span>
	);
}

function SkillDetails({ mode, entry, duration }: { mode: Mode; entry: Skill; duration: number }) {
	if (mode === "healing") {
		return (
			<span className="flex flex-col gap-2 px-2 pb-2 text-xs">
				<span className="grid grid-cols-2 gap-1.5">
					<Tile title="Moyenne par soin" value={compact(entry.amount / Math.max(entry.hits, 1))} />
					<Tile title="Soins" value={String(entry.hits)} />
				</span>
				<Meter title="Utiles" part={entry.effective} total={entry.amount} />
			</span>
		);
	}
	const { slots, tier } = specializations(entry.variants);
	return (
		<span className="flex flex-col gap-2 px-2 pb-2 text-xs">
			<span className="grid grid-cols-4 gap-1.5">
				<Tile title="Moyenne" value={compact(entry.amount / Math.max(entry.hits, 1))} />
				<Tile title="Coup max" value={compact(entry.max)} />
				<Tile title="Lancements" value={String(entry.casts)} />
				<Tile title="Par minute" value={(entry.casts / Math.max(duration / 60_000, 1 / 60)).toFixed(1)} />
			</span>
			<span className="flex flex-col gap-1">
				<Meter title="Critique" part={entry.crits} total={entry.hits} />
				<Meter title="Perfect" part={entry.perfect} total={entry.hits} />
				<Meter title="Puissant" part={entry.hard} total={entry.hits} />
				<Meter title="Dos" part={entry.back} total={entry.hits} />
			</span>
			{(slots.length > 0 || tier > 0) && (
				<span className="flex flex-wrap items-center gap-1">
					<span className="mr-1 text-muted-foreground">Spécialisations</span>
					{slots.map((slot) => (
						<Badge key={slot} variant="secondary" className="h-4 px-1.5 text-[10px]">
							Spé {slot}
						</Badge>
					))}
					{tier > 0 && (
						<Badge variant="outline" className="h-4 px-1.5 text-[10px]">
							Palier {tier}
						</Badge>
					)}
				</span>
			)}
		</span>
	);
}

function Uptimes({ title, uptimes, duration }: { title: string; uptimes: Uptime[]; duration: number }) {
	if (uptimes.length === 0) {
		return null;
	}
	return (
		<div className="flex flex-col gap-1">
			<span className="text-xs font-semibold text-muted-foreground">{title}</span>
			{uptimes.map((uptime) => (
				<div key={uptime.id} className="relative flex h-7 shrink-0 items-center gap-2 overflow-hidden rounded-md bg-muted/40 px-2 text-xs">
					<span className="absolute inset-y-0 left-0 bg-sky-400/20" style={{ width: `${Math.min((uptime.active / Math.max(duration, 1)) * 100, 100)}%` }} />
					<SkillIcon id={uptime.id} gameClass={Math.floor(uptime.id / 1_000_000) - 10} />
					<span className="relative min-w-0 flex-1 truncate">{skillName(uptime.id)}</span>
					<span className="relative w-10 text-right font-semibold tabular-nums">{percent(uptime.active, duration)}</span>
				</div>
			))}
		</div>
	);
}

function Comparison({ player, rival, mode, duration }: { player: Player; rival: Player; mode: Mode; duration: number }) {
	const mine = STATS[mode](player, duration);
	const theirs = STATS[mode](rival, duration);
	const ids = [...new Set([...ENTRIES[mode](player), ...ENTRIES[mode](rival)].map((entry) => entry.id))];
	const find = (target: Player, id: number) => ENTRIES[mode](target).find((entry) => entry.id === id)?.amount ?? 0;
	ids.sort((a, b) => Math.max(find(player, b), find(rival, b)) - Math.max(find(player, a), find(rival, a)));
	return (
		<div className="flex flex-col gap-1 text-xs">
			<div className="grid grid-cols-[1fr_5rem_5rem] gap-2 border-b pb-1 font-semibold text-muted-foreground">
				<span />
				<span className="truncate text-right">{player.name ?? CLASSES[player.class].name}</span>
				<span className="truncate text-right">{rival.name ?? CLASSES[rival.class].name}</span>
			</div>
			<div className="grid grid-cols-[1fr_5rem_5rem] gap-x-2 gap-y-1 tabular-nums">
				<span className="text-muted-foreground">{mode === "damage" ? "DPS" : mode === "healing" ? "Soins/s" : "Subis/s"}</span>
				<span className="text-right font-semibold">{compact(rate(player, mode))}</span>
				<span className="text-right font-semibold">{compact(rate(rival, mode))}</span>
				{mine.map(([name, value], index) => (
					<span key={name} className="contents">
						<span className="text-muted-foreground">{name}</span>
						<span className="text-right">{value}</span>
						<span className="text-right">{theirs[index][1]}</span>
					</span>
				))}
			</div>
			<div className="mt-2 grid grid-cols-[1fr_5rem_5rem] gap-x-2 gap-y-1 border-t pt-2 tabular-nums">
				{ids.map((id) => (
					<span key={id} className="contents">
						<span className="truncate">{mode === "taken" ? (npcName(id) ?? `#${id}`) : skillName(id)}</span>
						<span className="text-right">{compact(find(player, id))}</span>
						<span className="text-right">{compact(find(rival, id))}</span>
					</span>
				))}
			</div>
		</div>
	);
}

export function PlayerDetail({ player, players, duration, mode, onBack }: { player: Player; players: Player[]; duration: number; mode: Mode; onBack: () => void }) {
	const [expanded, setExpanded] = useState<number | null>(null);
	const [rivalId, setRivalId] = useState<number | null>(null);
	const { name, color } = CLASSES[player.class];
	const entries = ENTRIES[mode](player);
	const total = amount(player, mode);
	const top = entries[0]?.amount || 1;
	const rival = players.find((other) => other.id === rivalId && other.id !== player.id);
	return (
		<section className="flex min-h-0 flex-1 flex-col">
			<div className="flex shrink-0 items-center gap-2 border-b px-2 py-2">
				<Button variant="ghost" size="icon-sm" onClick={onBack}>
					<ChevronLeft />
				</Button>
				<ClassIcon gameClass={player.class} />
				<span className="truncate font-semibold">{player.name ?? name}</span>
				{player.own && <Badge className="h-4 px-1 text-[10px]">MOI</Badge>}
				{player.gear !== null && <span className="shrink-0 text-xs text-muted-foreground tabular-nums">GS {player.gear} · CP {compact(player.power ?? 0)}</span>}
				<span className="ml-auto text-sm tabular-nums">{compact(rate(player, mode))}/s</span>
			</div>
			<div className="grid shrink-0 grid-cols-3 gap-x-2 gap-y-1 border-b px-3 py-2 text-xs text-muted-foreground">
				{STATS[mode](player, duration).map(([name, value]) => (
					<span key={name}>
						{name} <b className="text-foreground tabular-nums">{value}</b>
					</span>
				))}
			</div>
			<div className="flex shrink-0 items-center gap-2 border-b px-3 py-1.5 text-xs text-muted-foreground">
				<span>Comparer avec</span>
				<select value={rivalId ?? ""} onChange={(event) => setRivalId(event.target.value ? Number(event.target.value) : null)} className="h-6 min-w-0 flex-1 rounded-md border bg-background px-1 text-foreground">
					<option value="">Personne</option>
					{players
						.filter((other) => other.id !== player.id)
						.map((other) => (
							<option key={other.id} value={other.id}>
								{other.name ?? CLASSES[other.class].name}
							</option>
						))}
				</select>
			</div>
			<div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
				{rival ? (
					<Comparison player={player} rival={rival} mode={mode} duration={duration} />
				) : (
					<>
						{entries.map((entry) => {
							const expandable = mode !== "taken" && entry.hits > 0;
							const open = expandable && expanded === entry.id;
							return (
								<button key={entry.id} type="button" disabled={!expandable} onClick={() => setExpanded(open ? null : entry.id)} className="flex shrink-0 flex-col overflow-hidden rounded-md bg-muted/40 text-left text-sm">
									<span className="relative flex h-8 w-full items-center gap-2 px-2">
										<span className="absolute inset-y-0 left-0 opacity-25" style={{ width: `${(entry.amount / top) * 100}%`, backgroundColor: mode === "taken" ? "#fb7185" : color }} />
										{mode === "taken" ? <Skull className="relative size-5 shrink-0 text-rose-400" /> : <SkillIcon id={entry.id} gameClass={player.class} />}
										<span className="relative min-w-0 flex-1 truncate">{label(mode, entry)}</span>
										<span className="relative w-14 text-right font-semibold tabular-nums">{compact(entry.amount)}</span>
										<span className="relative w-10 text-right text-muted-foreground tabular-nums">{percent(entry.amount, total)}</span>
										<span className="relative w-16 text-right text-xs text-muted-foreground tabular-nums">{details(mode, entry)}</span>
										{expandable && <ChevronDown className={cn("relative size-3.5 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} />}
									</span>
									{open && <SkillDetails mode={mode} entry={entry} duration={duration} />}
								</button>
							);
						})}
						{mode === "damage" && (
							<div className="mt-2 flex flex-col gap-3">
								<Uptimes title="Buffs reçus" uptimes={player.buffs} duration={duration} />
								<Uptimes title="Debuffs sur la cible" uptimes={player.debuffs} duration={duration} />
							</div>
						)}
					</>
				)}
			</div>
		</section>
	);
}