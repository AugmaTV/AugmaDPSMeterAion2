import daevanion from "@/data/daevanion.json";
import itemDetails from "@/data/item-details.json";
import itemStats from "@/data/item-stats.json";
import items from "@/data/items.json";
import npcs from "@/data/npcs.json";
import extra from "@/data/skills-extra.json";
import official from "@/data/skills.json";
import { dictionary, language } from "@/lib/i18n";
import type { Stone } from "@/lib/meter";

type SkillEntry = {
	fr?: string;
	en?: string;
	icon?: string;
};

type ItemEntry = {
	fr?: string;
	en?: string;
	icon?: string;
	grade?: string;
};

type StatEntry = {
	id: string;
	fr: string;
	en: string;
	scale: number;
	percent: boolean;
};

type DetailStat = {
	stat: string;
	min?: string | null;
	value: string;
};

export type ItemDetail = {
	level: number | null;
	category: { fr: string | null; en: string | null };
	max: number;
	main: DetailStat[];
	bonus?: Record<string, string[]>;
	over?: Record<string, DetailStat[]>;
	pool?: DetailStat[];
	fixed?: DetailStat[];
	set?: string;
};

type SetDetail = {
	fr: string | null;
	en: string | null;
	items: number[];
	bonuses: { degree: number; fr: string[] | null; en: string[] | null }[];
};

export type DaevanionNode = {
	row: number;
	col: number;
	type: string;
	grade: number;
	fr: string[];
	en: string[];
};

const SKILLS: Record<string, SkillEntry> = { ...extra, ...official };
const ITEMS: Record<string, ItemEntry> = items;
const STATS: Record<string, StatEntry> = itemStats.stats;
const STONES: Record<string, { grade: string; value: number }> = itemStats.stones;
const GODSTONES: Record<string, { fr?: string; en: string }> = itemStats.godstones;
const DETAILS: Record<string, ItemDetail> = itemDetails.items;
const DETAIL_NAMES: Record<string, { fr?: string; en?: string }> = itemDetails.names;
const SETS: Record<string, SetDetail> = itemDetails.sets;
const NODES: Record<string, DaevanionNode> = daevanion.nodes;
const NODE_BOARD = 10_000;
const EFFECT = /^(.*?)\s*([+-]\d+(?:[.,]\d+)?)(%?)$/;
const NPCS: Record<string, string> = npcs;
const ICON_URL = "https://assets.playnccdn.com/static-aion2-gamedata/resources/";
const DRAIN_SKILL = 1;
const BASIC_ATTACK_START = 100_000;
const BASIC_ATTACK_END = 101_000;
const CLASS_SKILLS_START = 11_000_000;
const CLASS_SKILLS_END = 20_000_000;
const ITEM_CATEGORY = 100_000;
const BOARD_INDEX = 10;
const SPECIES: Record<number, string> = { 2: "Intellia", 3: "Bestia", 4: "Natura", 5: "Varius", 6: "Singulia" };
const GRADE_NAMES = ["Common", "Rare", "Legend", "Unique", "Epic"];
const DAEVANION_BOARDS: Record<number, { name: string; total: number }> = {
	1: { name: "Nezekan", total: 88 },
	2: { name: "Zikel", total: 88 },
	3: { name: "Vaizel", total: 88 },
	4: { name: "Triniel", total: 116 },
	6: { name: "Azphel", total: 152 },
};
const WEAPON_CATEGORIES_END = 1150;
const ITEM_ORDER = [1150, 2103, 2104, 2101, 2102, 2105, 2106, 2107, 2152, 3101, 3102, 3103, 3104, 3105, 3110, 3111, 3109, 8101, 8102, 8103, 8104, 8105, 8106, 8107, 8108, 8109, 8110];

export function skillName(id: number) {
	if (id === DRAIN_SKILL) {
		return dictionary().skills.drain;
	}
	if (id >= BASIC_ATTACK_START && id < BASIC_ATTACK_END) {
		return dictionary().skills.basicAttack;
	}
	const entry = SKILLS[id];
	return (language() === "fr" ? (entry?.fr ?? entry?.en) : (entry?.en ?? entry?.fr)) ?? `#${id}`;
}

export function skillKnown(id: number) {
	return id in SKILLS;
}

export function skillIcon(id: number) {
	const icon = SKILLS[id]?.icon;
	return icon ? `${ICON_URL}${icon}.png` : null;
}

export function itemName(id: number) {
	const entry = ITEMS[id];
	return (language() === "fr" ? (entry?.fr ?? entry?.en) : (entry?.en ?? entry?.fr)) ?? `#${id}`;
}

export function itemIcon(id: number) {
	const icon = ITEMS[id]?.icon;
	return icon ? `${ICON_URL}${icon}.png` : null;
}

export function itemGrade(id: number) {
	return ITEMS[id]?.grade ?? null;
}

export function itemOrder(id: number) {
	const category = Math.floor(id / ITEM_CATEGORY);
	if (category < WEAPON_CATEGORIES_END) {
		return -1;
	}
	const index = ITEM_ORDER.indexOf(category);
	return index === -1 ? ITEM_ORDER.length : index;
}

export function daevanionBoard(id: number) {
	return DAEVANION_BOARDS[id % BOARD_INDEX] ?? null;
}

export function statName(code: number) {
	const entry = STATS[code];
	return entry ? (language() === "fr" ? entry.fr : entry.en) : `#${code}`;
}

export function statId(code: number) {
	return STATS[code]?.id ?? null;
}

export function itemDetail(id: number) {
	return DETAILS[id] ?? null;
}

export function detailName(stat: string) {
	const entry = DETAIL_NAMES[stat];
	return (language() === "fr" ? (entry?.fr ?? entry?.en) : (entry?.en ?? entry?.fr)) ?? stat;
}

export function localized(entry: { fr: string | null; en: string | null }) {
	return language() === "fr" ? (entry.fr ?? entry.en) : (entry.en ?? entry.fr);
}

export function itemSet(key: string) {
	return SETS[key] ?? null;
}

export function localizedValue(text: string) {
	const match = /^(-?[\d.]+)(%?)$/.exec(text);
	return match ? `${Number(match[1]).toLocaleString(language(), { maximumFractionDigits: 2 })}${match[2]}` : text;
}

export function statValue(code: number, value: number) {
	const entry = STATS[code];
	const text = (value / (entry?.scale ?? 1)).toLocaleString(language(), { maximumFractionDigits: 2 });
	return entry?.percent ? `${text}%` : text;
}

export function stoneValue(stone: Stone) {
	return STONES[`${stone.item}:${stone.rank}:${stone.stat}`] ?? null;
}

export function godstoneName(id: number) {
	const entry = GODSTONES[id];
	return entry ? (language() === "fr" ? (entry.fr ?? entry.en) : entry.en) : `#${id}`;
}

export function speciesName(id: number) {
	return SPECIES[id] ?? `#${id}`;
}

export function gradeName(grade: number) {
	return GRADE_NAMES[grade - 1] ?? null;
}

export function daevanionNodes(board: number) {
	return Object.entries(NODES)
		.filter(([id]) => Math.floor(Number(id) / NODE_BOARD) === board)
		.map(([id, node]) => ({ id: Number(id), ...node }));
}

export function nodeEffects(node: DaevanionNode) {
	return language() === "fr" || node.en.length === 0 ? node.fr : node.en;
}

export function daevanionTotals(opened: number[], type: string) {
	const totals = new Map<string, { name: string; value: number; percent: string }>();
	for (const id of opened) {
		const node = NODES[id];
		if (node?.type !== type) {
			continue;
		}
		for (const effect of nodeEffects(node)) {
			const match = EFFECT.exec(effect);
			if (match) {
				const entry = totals.get(match[1] + match[3]) ?? { name: match[1], value: 0, percent: match[3] };
				entry.value += Number(match[2].replace(",", "."));
				totals.set(match[1] + match[3], entry);
			}
		}
	}
	return [...totals.values()].map((entry) => ({ name: entry.name, value: `${entry.value.toLocaleString(language(), { maximumFractionDigits: 2 })}${entry.percent}` }));
}

export function npcName(id: number | null) {
	return id === null ? null : (NPCS[id] ?? null);
}

export function specializations(variants: number[]) {
	const codes = variants.filter((variant) => variant >= CLASS_SKILLS_START && variant < CLASS_SKILLS_END);
	return {
		slots: [...new Set(codes.flatMap((code) => [...String(Math.floor((code % 10_000) / 10))].filter((slot) => slot !== "0")))].sort(),
		tier: Math.max(0, ...codes.map((code) => code % 10)),
	};
}