import { clock, compact, percent } from "@/lib/format";
import { npcName } from "@/lib/game-data";
import { dictionary } from "@/lib/i18n";
import { ranking, totalRate, type Snapshot } from "@/lib/meter";

export function summary(snapshot: Snapshot, title: string | null) {
	const t = dictionary();
	const players = ranking(snapshot.players, "damage");
	const heading = title ?? npcName(snapshot.boss?.npc ?? null) ?? t.summary.fight;
	return [
		`${heading} — ${clock(snapshot.duration)} — ${compact(totalRate(snapshot.players, "damage"))} DPS`,
		...players.map((player, index) => `${index + 1}. ${player.name ?? t.classes[player.class]} (${t.classes[player.class]}) — ${compact(player.dps)}/s — ${compact(player.damage)} (${percent(player.damage, snapshot.total)})`),
	].join("\n");
}