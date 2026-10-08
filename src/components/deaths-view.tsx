import { useState } from "react";
import { HeartPulse, Skull } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ClassIcon } from "@/components/class-icon";
import { CLASSES } from "@/lib/classes";
import { clock, compact } from "@/lib/format";
import { npcName } from "@/lib/game-data";
import type { Snapshot } from "@/lib/meter";

export function DeathsView({ snapshot }: { snapshot: Snapshot | null }) {
	const [expanded, setExpanded] = useState<number | null>(null);
	const players = (snapshot?.players ?? []).filter((player) => player.deaths > 0 || player.resurrections > 0).sort((a, b) => b.deaths - a.deaths || b.resurrections - a.resurrections);
	return (
		<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
			{players.length === 0 && <p className="m-auto text-sm text-muted-foreground">Aucune mort</p>}
			{players.map((player) => (
				<button key={player.id} type="button" onClick={() => setExpanded(expanded === player.id ? null : player.id)} className="flex shrink-0 flex-col rounded-md bg-muted/40 text-left text-sm">
					<span className="flex h-9 w-full items-center gap-2 px-2">
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
					</span>
					{expanded === player.id &&
						player.recaps.map((recap, index) => (
							<span key={index} className="flex flex-col gap-0.5 border-t px-2 py-1.5 text-xs">
								<span className="font-semibold">Mort à {clock(recap.at)}</span>
								{recap.blows.length === 0 && <span className="text-muted-foreground">Aucun coup reçu enregistré</span>}
								{recap.blows.map((blow, position) => (
									<span key={position} className="flex items-center gap-2 text-muted-foreground tabular-nums">
										<span className="w-12 shrink-0 text-right">{((blow.at - recap.at) / 1000).toFixed(1)} s</span>
										<span className="min-w-0 flex-1 truncate text-foreground">{npcName(blow.npc) ?? "Monstre inconnu"}</span>
										<span className="shrink-0">#{blow.skill}</span>
										<span className="w-12 shrink-0 text-right font-semibold text-foreground">{compact(blow.amount)}</span>
									</span>
								))}
							</span>
						))}
				</button>
			))}
		</section>
	);
}