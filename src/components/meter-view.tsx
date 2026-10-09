import { PlayerDetail } from "@/components/player-detail";
import { PlayerRow } from "@/components/player-row";
import { useDictionary } from "@/lib/i18n";
import { amount, ranking, total, type Mode, type Snapshot } from "@/lib/meter";

export function MeterView({ snapshot, mode, selected, onSelect }: { snapshot: Snapshot | null; mode: Mode; selected: number | null; onSelect: (id: number | null) => void }) {
	const t = useDictionary();
	const players = ranking(snapshot?.players ?? [], mode);
	const player = snapshot?.players.find((player) => player.id === selected);
	const top = players[0] ? amount(players[0], mode) : 1;
	if (player) {
		return <PlayerDetail player={player} players={snapshot?.players ?? []} duration={snapshot?.duration ?? 0} mode={mode} onBack={() => onSelect(null)} />;
	}
	return (
		<section className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
			{snapshot?.status === "unavailable" && <p className="m-auto p-4 text-center text-sm text-muted-foreground">{t.status.unavailable}</p>}
			{snapshot?.status !== "unavailable" && players.length === 0 && <p className="m-auto text-sm text-muted-foreground">{snapshot?.players.length ? t.status.empty[mode] : snapshot?.status === "live" ? t.status.waitingFight : t.status.waitingGame}</p>}
			{players.map((player) => (
				<PlayerRow key={player.id} player={player} mode={mode} top={top} total={snapshot ? total(snapshot, mode) : 0} onSelect={() => onSelect(player.id)} />
			))}
		</section>
	);
}