export type Status = "unavailable" | "waiting" | "live";

export type Mode = "damage" | "healing" | "taken";

export type Skill = {
	id: number;
	amount: number;
	hits: number;
	crits: number;
};

export type Player = {
	id: number;
	name: string | null;
	class: number;
	damage: number;
	dps: number;
	hits: number;
	crits: number;
	healing: number;
	hps: number;
	taken: number;
	dtps: number;
	takenHits: number;
	own: boolean;
	skills: Skill[];
	heals: Skill[];
	sources: Skill[];
};

export type Boss = {
	npc: number | null;
	hp: number;
	max: number;
	estimated: boolean;
	dead: boolean;
};

export type Snapshot = {
	status: Status;
	duration: number;
	total: number;
	totalHealing: number;
	totalTaken: number;
	boss: Boss | null;
	players: Player[];
};

export type OverlayState = {
	open: boolean;
	locked: boolean;
};

export type SessionSummary = {
	id: number;
	name: string | null;
	boss: number | null;
	start: number;
	duration: number;
	fights: number;
	dps: number;
	locked: boolean;
};

export type SessionView = {
	fights: { boss: number | null; duration: number }[];
	snapshot: Snapshot;
};

export const MODES: Mode[] = ["damage", "healing", "taken"];

export const TITLES: Record<Mode, string> = {
	damage: "Dégâts",
	healing: "Soins",
	taken: "Subis",
};

const AMOUNTS: Record<Mode, (player: Player) => number> = {
	damage: (player) => player.damage,
	healing: (player) => player.healing,
	taken: (player) => player.taken,
};

const RATES: Record<Mode, (player: Player) => number> = {
	damage: (player) => player.dps,
	healing: (player) => player.hps,
	taken: (player) => player.dtps,
};

const TOTALS: Record<Mode, (snapshot: Snapshot) => number> = {
	damage: (snapshot) => snapshot.total,
	healing: (snapshot) => snapshot.totalHealing,
	taken: (snapshot) => snapshot.totalTaken,
};

export function amount(player: Player, mode: Mode) {
	return AMOUNTS[mode](player);
}

export function rate(player: Player, mode: Mode) {
	return RATES[mode](player);
}

export function total(snapshot: Snapshot, mode: Mode) {
	return TOTALS[mode](snapshot);
}

export function totalRate(players: Player[], mode: Mode) {
	return players.reduce((sum, player) => sum + rate(player, mode), 0);
}

export function ranking(players: Player[], mode: Mode) {
	return players.filter((player) => amount(player, mode) > 0).sort((a, b) => amount(b, mode) - amount(a, mode));
}