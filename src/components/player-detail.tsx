import { ChevronLeft } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ClassIcon } from "@/components/class-icon";
import { SkillIcon } from "@/components/skill-icon";
import { CLASSES } from "@/lib/classes";
import { compact, percent } from "@/lib/format";
import { skillName } from "@/lib/game-data";
import { amount, rate, type Mode, type Player } from "@/lib/meter";

export function PlayerDetail({ player, mode, onBack }: { player: Player; mode: Mode; onBack: () => void }) {
	const { name, color } = CLASSES[player.class];
	const skills = mode === "damage" ? player.skills : player.heals;
	const total = amount(player, mode);
	const top = skills[0]?.amount || 1;
	return (
		<section className="flex min-h-0 flex-1 flex-col">
			<div className="flex shrink-0 items-center gap-2 border-b px-2 py-2">
				<Button variant="ghost" size="icon-sm" onClick={onBack}>
					<ChevronLeft />
				</Button>
				<ClassIcon gameClass={player.class} />
				<span className="truncate font-semibold">{player.name ?? name}</span>
				{player.own && <Badge className="h-4 px-1 text-[10px]">MOI</Badge>}
				<span className="ml-auto text-sm tabular-nums">{compact(rate(player, mode))}/s</span>
			</div>
			<div className="grid shrink-0 grid-cols-3 gap-2 border-b px-3 py-2 text-xs text-muted-foreground">
				<span>Total <b className="text-foreground tabular-nums">{compact(total)}</b></span>
				{mode === "damage" ? (
					<>
						<span>Coups <b className="text-foreground tabular-nums">{player.hits}</b></span>
						<span>Crit <b className="text-foreground tabular-nums">{percent(player.crits, player.hits)}</b></span>
					</>
				) : (
					<span>Soins <b className="text-foreground tabular-nums">{skills.reduce((sum, skill) => sum + skill.hits, 0)}</b></span>
				)}
			</div>
			<div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
				{skills.map((skill) => (
					<div key={skill.id} className="relative flex h-8 shrink-0 items-center gap-2 overflow-hidden rounded-md bg-muted/40 px-2 text-sm">
						<span className="absolute inset-y-0 left-0 opacity-25" style={{ width: `${(skill.amount / top) * 100}%`, backgroundColor: color }} />
						<SkillIcon id={skill.id} gameClass={player.class} />
						<span className="relative min-w-0 flex-1 truncate">{skillName(skill.id)}</span>
						<span className="relative w-14 text-right font-semibold tabular-nums">{compact(skill.amount)}</span>
						<span className="relative w-10 text-right text-muted-foreground tabular-nums">{percent(skill.amount, total)}</span>
						<span className="relative w-16 text-right text-xs text-muted-foreground tabular-nums">{mode === "healing" ? `${skill.hits}×` : skill.hits ? `${skill.hits} · ${percent(skill.crits, skill.hits)}` : "DoT"}</span>
					</div>
				))}
			</div>
		</section>
	);
}