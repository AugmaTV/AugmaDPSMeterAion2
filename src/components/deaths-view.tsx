import { HeartPulse, Skull } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ClassIcon } from "@/components/class-icon";
import { CLASSES } from "@/lib/classes";
import type { Snapshot } from "@/lib/meter";

export function DeathsView({ snapshot }: { snapshot: Snapshot | null }) {
	const players = (snapshot?.players ?? []).filter((player) => player.deaths > 0 || player.resurrections > 0).sort((a, b) => b.deaths - a.deaths || b.resurrections - a.resurrections);
	return (
		<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
			{players.length === 0 && <p className="m-auto text-sm text-muted-foreground">Aucune mort</p>}
			{players.map((player) => (
				<div key={player.id} className="flex h-9 shrink-0 items-center gap-2 rounded-md bg-muted/40 px-2 text-sm">
					<ClassIcon gameClass={player.class} />
					<span className="flex min-w-0 flex-1 items-center gap-1.5">
						<span className="truncate font-medium">{player.name ?? CLASSES[player.class].name}</span>
						{player.own && <Badge className="h-4 px-1 text-[10px]">MOI</Badge>}
					</span>
					<span className="flex items-center gap-1 font-semibold tabular-nums" title="Morts">
						<Skull className="size-4 text-rose-400" />
						{player.deaths}
					</span>
					{player.revived > 0 && (
						<span className="text-xs text-emerald-400 tabular-nums" title="Ressuscité par un Clerc">
							{player.revived} rés.
						</span>
					)}
					{player.resurrections > 0 && (
						<span className="flex items-center gap-1 font-semibold text-emerald-400 tabular-nums" title="Résurrections effectuées">
							<HeartPulse className="size-4" />
							{player.resurrections}
						</span>
					)}
				</div>
			))}
		</section>
	);
}