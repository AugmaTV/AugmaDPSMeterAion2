import npcs from "@/data/npcs.json";
import extra from "@/data/skills-extra.json";
import official from "@/data/skills.json";

type SkillEntry = {
	fr?: string;
	en?: string;
	icon?: string;
};

const SKILLS: Record<string, SkillEntry> = { ...extra, ...official };
const NPCS: Record<string, string> = npcs;
const ICON_URL = "https://assets.playnccdn.com/static-aion2-gamedata/resources/";
const DRAIN_SKILL = 1;
const BASIC_ATTACK_START = 100_000;
const BASIC_ATTACK_END = 101_000;

export function skillName(id: number) {
	if (id === DRAIN_SKILL) {
		return "Vol de vie";
	}
	if (id >= BASIC_ATTACK_START && id < BASIC_ATTACK_END) {
		return "Attaque de base";
	}
	const entry = SKILLS[id];
	return entry?.fr ?? entry?.en ?? `#${id}`;
}

export function skillIcon(id: number) {
	const icon = SKILLS[id]?.icon;
	return icon ? `${ICON_URL}${icon}.png` : null;
}

export function npcName(id: number | null) {
	return id === null ? null : (NPCS[id] ?? null);
}