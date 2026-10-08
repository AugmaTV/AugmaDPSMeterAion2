import { CLASSES } from "@/lib/classes";
import { clock, compact, percent } from "@/lib/format";
import { npcName } from "@/lib/game-data";
import { ranking, totalRate, type Snapshot } from "@/lib/meter";

export function summary(snapshot: Snapshot, title: string | null) {
	const players = ranking(snapshot.players, "damage");
	const heading = title ?? npcName(snapshot.boss?.npc ?? null) ?? "Combat";
	return [
		`${heading} — ${clock(snapshot.duration)} — ${compact(totalRate(snapshot.players, "damage"))} DPS`,
		...players.map((player, index) => `${index + 1}. ${player.name ?? CLASSES[player.class].name} (${CLASSES[player.class].name}) — ${compact(player.dps)}/s — ${compact(player.damage)} (${percent(player.damage, snapshot.total)})`),
	].join("\n");
}