import { ChevronLeft, Skull } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ClassIcon } from "@/components/class-icon";
import { SkillIcon } from "@/components/skill-icon";
import { CLASSES } from "@/lib/classes";
import { compact, percent } from "@/lib/format";
import { npcName, skillName } from "@/lib/game-data";
import { amount, rate, type Mode, type Player, type Skill } from "@/lib/meter";

const ENTRIES: Record<Mode, (player: Player) => Skill[]> = {
	damage: (player) => player.skills,
	healing: (player) => player.heals,
	taken: (player) => player.sources,
};

function details(mode: Mode, entry: Skill) {
	if (mode === "healing") {
		return `${entry.hits}×`;
	}
	if (!entry.hits) {
		return "DoT";
	}
	return mode === "damage" ? `${entry.hits} · ${percent(entry.crits, entry.hits)}` : `${entry.hits} coups`;
}

export function PlayerDetail({ player, mode, onBack }: { player: Player; mode: Mode; onBack: () => void }) {
	const { name, color } = CLASSES[player.class];
	const entries = ENTRIES[mode](player);
	const total = amount(player, mode);
	const top = entries[0]?.amount || 1;
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
			<div className="grid shrink-0 grid-cols-3 gap-2 border-b px-3 py-2 text-xs text-muted-foreground">
				<span>Total <b className="text-foreground tabular-nums">{compact(total)}</b></span>
				{mode === "damage" && (
					<>
						<span>Coups <b className="text-foreground tabular-nums">{player.hits}</b></span>
						<span>Crit <b className="text-foreground tabular-nums">{percent(player.crits, player.hits)}</b></span>
					</>
				)}
				{mode === "healing" && <span>Soins <b className="text-foreground tabular-nums">{entries.reduce((sum, entry) => sum + entry.hits, 0)}</b></span>}
				{mode === "taken" && <span>Coups reçus <b className="text-foreground tabular-nums">{player.takenHits}</b></span>}
			</div>
			<div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
				{entries.map((entry) => (
					<div key={entry.id} className="relative flex h-8 shrink-0 items-center gap-2 overflow-hidden rounded-md bg-muted/40 px-2 text-sm">
						<span className="absolute inset-y-0 left-0 opacity-25" style={{ width: `${(entry.amount / top) * 100}%`, backgroundColor: mode === "taken" ? "#fb7185" : color }} />
						{mode === "taken" ? <Skull className="relative size-5 shrink-0 text-rose-400" /> : <SkillIcon id={entry.id} gameClass={player.class} />}
						<span className="relative min-w-0 flex-1 truncate">{mode === "taken" ? (npcName(entry.id) ?? (entry.id ? `#${entry.id}` : "Inconnu")) : skillName(entry.id)}</span>
						<span className="relative w-14 text-right font-semibold tabular-nums">{compact(entry.amount)}</span>
						<span className="relative w-10 text-right text-muted-foreground tabular-nums">{percent(entry.amount, total)}</span>
						<span className="relative w-16 text-right text-xs text-muted-foreground tabular-nums">{details(mode, entry)}</span>
					</div>
				))}
			</div>
		</section>
	);
}