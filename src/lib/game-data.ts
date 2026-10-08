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
const CLASS_SKILLS_START = 11_000_000;
const CLASS_SKILLS_END = 20_000_000;

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

export function specialization(variants: number[]) {
	const codes = variants.filter((variant) => variant >= CLASS_SKILLS_START && variant < CLASS_SKILLS_END);
	const slots = [...new Set(codes.flatMap((code) => [...String(Math.floor((code % 10_000) / 10))].filter((slot) => slot !== "0")))].sort();
	const tier = Math.max(0, ...codes.map((code) => code % 10));
	if (slots.length === 0 && tier === 0) {
		return null;
	}
	return [slots.length ? `Spé ${slots.join("·")}` : null, tier ? `palier ${tier}` : null].filter(Boolean).join(" · ");
}