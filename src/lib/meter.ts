export type Status = "unavailable" | "waiting" | "live";

export type Mode = "damage" | "healing";

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
	own: boolean;
	skills: Skill[];
	heals: Skill[];
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
	boss: Boss | null;
	players: Player[];
};

export type OverlayState = {
	open: boolean;
	locked: boolean;
};

export function amount(player: Player, mode: Mode) {
	return mode === "damage" ? player.damage : player.healing;
}

export function rate(player: Player, mode: Mode) {
	return mode === "damage" ? player.dps : player.hps;
}

export function ranking(players: Player[], mode: Mode) {
	return players.filter((player) => amount(player, mode) > 0).sort((a, b) => amount(b, mode) - amount(a, mode));
}