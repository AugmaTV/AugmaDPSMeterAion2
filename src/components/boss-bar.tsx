import { Skull } from "lucide-react";
import { compact } from "@/lib/format";
import { npcName } from "@/lib/game-data";
import { useDictionary } from "@/lib/i18n";
import type { Boss } from "@/lib/meter";
import { cn } from "@/lib/utils";

export function BossBar({ boss, compactView = false }: { boss: Boss; compactView?: boolean }) {
	const t = useDictionary();
	const ratio = boss.dead ? 0 : Math.min(boss.hp / (boss.max || 1), 1);
	const name = npcName(boss.npc) ?? t.boss;
	return (
		<div className={cn("flex shrink-0 flex-col gap-1", compactView ? "px-2 py-1 text-[11px]" : "border-b px-3 py-2 text-xs")}>
			<div className="flex items-center gap-1.5">
				<Skull className={cn("shrink-0 text-rose-400", compactView ? "size-3" : "size-3.5")} />
				<span className="truncate font-semibold">{name}</span>
				<span className="ml-auto text-muted-foreground tabular-nums">
					{compact(boss.hp)} / {boss.estimated ? "~" : ""}{compact(boss.max)}
				</span>
				<span className="w-10 text-right font-semibold text-rose-400 tabular-nums">{Math.round(ratio * 100)}%</span>
			</div>
			<div className={cn("w-full overflow-hidden rounded-full bg-white/10", compactView ? "h-1" : "h-1.5")}>
				<div className="h-full rounded-full bg-rose-400 transition-[width]" style={{ width: `${ratio * 100}%` }} />
			</div>
		</div>
	);
}