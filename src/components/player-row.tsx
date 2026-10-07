import { Badge } from "@/components/ui/badge";
import { ClassIcon } from "@/components/class-icon";
import { CLASSES } from "@/lib/classes";
import { compact, percent } from "@/lib/format";
import { amount, rate, type Mode, type Player } from "@/lib/meter";

export function PlayerRow({ player, mode, top, total, onSelect }: { player: Player; mode: Mode; top: number; total: number; onSelect: () => void }) {
	const { name, color } = CLASSES[player.class];
	const value = amount(player, mode);
	return (
		<button type="button" onClick={onSelect} className="relative flex h-9 shrink-0 items-center gap-2 overflow-hidden rounded-md bg-muted/40 px-2 text-left text-sm transition-colors hover:bg-muted">
			<span className="absolute inset-y-0 left-0 opacity-30" style={{ width: `${(value / top) * 100}%`, backgroundColor: color }} />
			<ClassIcon gameClass={player.class} className="relative" />
			<span className="relative flex min-w-0 flex-1 items-center gap-1.5">
				<span className="truncate font-medium">{player.name ?? name}</span>
				{player.own && <Badge className="h-4 px-1 text-[10px]">MOI</Badge>}
				{player.gear !== null && <span className="shrink-0 text-[10px] text-muted-foreground tabular-nums">GS {player.gear}</span>}
			</span>
			<span className="relative w-14 text-right font-semibold tabular-nums">{compact(rate(player, mode))}</span>
			<span className="relative w-10 text-right text-muted-foreground tabular-nums">{percent(value, total)}</span>
			<span className="relative w-14 text-right text-muted-foreground tabular-nums">{compact(value)}</span>
		</button>
	);
}