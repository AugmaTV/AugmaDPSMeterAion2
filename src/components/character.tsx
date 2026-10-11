import { useEffect, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Gem } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ClassIcon } from "@/components/class-icon";
import { GearIcon, gradeBorder, gradeText } from "@/components/gear-icon";
import { SkillIcon } from "@/components/skill-icon";
import { daevanionBoard, daevanionNodes, daevanionTotals, detailName, godstoneName, itemDetail, itemGrade, itemName, itemOrder, itemSet, localized, localizedValue, nodeEffects, skillKnown, skillName, speciesName, statId, statName, statValue, stoneValue } from "@/lib/game-data";
import { language, useDictionary } from "@/lib/i18n";
import type { Board, Gear, Profile, SkillLevel, Species } from "@/lib/meter";
import { cn } from "@/lib/utils";

function Line({ title, children }: { title: string; children: ReactNode }) {
	return (
		<span className="flex gap-1.5">
			<span className="w-20 shrink-0 text-[10px] leading-4 text-muted-foreground">{title}</span>
			<span className="flex min-w-0 flex-1 flex-wrap gap-x-2.5 gap-y-0.5">{children}</span>
		</span>
	);
}

function GearDetail({ gear }: { gear: Gear }) {
	const t = useDictionary();
	const detail = itemDetail(gear.id);
	const capped = Math.min(gear.enchant, detail?.max ?? 0);
	const exceed = detail?.over?.[gear.enchant] ?? [];
	const pool = new Map((detail?.pool ?? []).map((entry) => [entry.stat, entry]));
	const category = detail ? localized(detail.category) : null;
	return (
		<div className="flex shrink-0 flex-col gap-1 rounded-md bg-muted/40 p-1.5">
			<span className="flex min-w-0 items-center gap-1.5">
				<GearIcon gear={gear} />
				<span className="flex min-w-0 flex-1 flex-col">
					<span className={cn("truncate", gradeText(itemGrade(gear.id)))}>{itemName(gear.id)}</span>
					{detail && <span className="truncate text-[10px] text-muted-foreground">{[category, detail.level ? t.detail.itemLevel(detail.level) : null].filter(Boolean).join(" · ")}</span>}
				</span>
				<span className="shrink-0 pr-1 font-semibold tabular-nums">+{gear.enchant}</span>
			</span>
			<span className="flex flex-col gap-1">
				{detail && detail.main.length > 0 && (
					<Line title={t.detail.mainStats}>
						{detail.main.map((stat) => {
							const bonus = capped > 0 ? detail.bonus?.[stat.stat]?.[capped - 1] : undefined;
							const shown = bonus && parseFloat(bonus) !== 0;
							return (
								<span key={stat.stat}>
									<span className="text-muted-foreground">{detailName(stat.stat)}</span>{" "}
									<b className="tabular-nums">{stat.min ? `${localizedValue(stat.min)}–${localizedValue(stat.value)}` : localizedValue(stat.value)}</b>
									{shown && <b className="ml-0.5 text-emerald-400 tabular-nums">+{localizedValue(bonus)}</b>}
								</span>
							);
						})}
					</Line>
				)}
				{exceed.length > 0 && (
					<Line title={t.detail.exceed}>
						{exceed.map((stat) => (
							<span key={stat.stat}>
								<span className="text-muted-foreground">{detailName(stat.stat)}</span> <b className="text-violet-400 tabular-nums">+{localizedValue(stat.value)}</b>
							</span>
						))}
					</Line>
				)}
				{gear.bonds.length > 0 && (
					<Line title={t.detail.bound}>
						{gear.bonds.map((bond, position) => {
							const range = pool.get(statId(bond.stat) ?? "");
							return (
								<span key={position}>
									<span className="text-muted-foreground">{statName(bond.stat)}</span> <b className="tabular-nums">{statValue(bond.stat, bond.value)}</b>
									{range?.min && <span className="ml-0.5 text-[10px] text-muted-foreground tabular-nums">({localizedValue(range.min)}–{localizedValue(range.value)})</span>}
								</span>
							);
						})}
					</Line>
				)}
				{gear.bonds.length === 0 && detail?.fixed && (
					<Line title={t.detail.fixedStats}>
						{detail.fixed.map((stat) => (
							<span key={stat.stat}>
								<span className="text-muted-foreground">{detailName(stat.stat)}</span> <b className="tabular-nums">{localizedValue(stat.value)}</b>
							</span>
						))}
					</Line>
				)}
				{gear.skills.length > 0 && (
					<Line title={t.detail.arcanaSkills}>
						{gear.skills.map((skill) => (
							<span key={skill.id}>
								{skillName(skill.id)} <span className="text-[10px] text-muted-foreground tabular-nums">{t.detail.level(skill.level)}</span>
							</span>
						))}
					</Line>
				)}
				{gear.stones.length > 0 && (
					<span className="flex flex-wrap gap-1 pl-21.5">
						{gear.stones.map((stone, position) => {
							const known = stoneValue(stone);
							return (
								<span key={position} className={cn("rounded border px-1 text-[10px]", gradeBorder(known?.grade ?? null), gradeText(known?.grade ?? null))}>
									{statName(stone.stat)}
									{known && <b className="ml-1 tabular-nums">+{known.value}</b>}
								</span>
							);
						})}
					</span>
				)}
				{gear.godstone !== null && (
					<span className="flex items-center gap-1 pl-21.5 text-[10px] text-amber-300">
						<Gem className="size-3 shrink-0" />
						{godstoneName(gear.godstone)}
					</span>
				)}
			</span>
		</div>
	);
}

function Sets({ equipment }: { equipment: Gear[] }) {
	const t = useDictionary();
	const counts = new Map<string, number>();
	for (const gear of equipment) {
		const key = itemDetail(gear.id)?.set;
		if (key) {
			counts.set(key, (counts.get(key) ?? 0) + 1);
		}
	}
	if (counts.size === 0) {
		return null;
	}
	return (
		<div className="flex shrink-0 flex-col gap-1 rounded-md bg-muted/40 p-1.5">
			<span className="text-[10px] font-semibold text-muted-foreground">{t.detail.sets}</span>
			{[...counts].map(([key, count]) => {
				const set = itemSet(key);
				if (!set) {
					return null;
				}
				return (
					<span key={key} className="flex flex-col gap-0.5">
						<span className="font-semibold">
							{localized(set)} <span className="text-[10px] font-normal text-muted-foreground">{t.detail.pieces(count)}</span>
						</span>
						{set.bonuses.map((bonus) => (
							<span key={bonus.degree} className={cn("text-[10px]", bonus.degree <= count ? "text-emerald-400" : "text-muted-foreground")}>
								{t.detail.pieces(bonus.degree)} : {(language() === "fr" ? (bonus.fr ?? bonus.en) : (bonus.en ?? bonus.fr))?.join(" · ")}
							</span>
						))}
					</span>
				);
			})}
		</div>
	);
}

const CELLS = ["bg-violet-400", "bg-neutral-300", "bg-emerald-400", "bg-sky-400", "bg-amber-400", "bg-fuchsia-400"];

const NODE_EFFECT = /^(.*?)\s*([+-]\d+(?:[.,]\d+)?%?)$/;

function nodeLabel(effects: string[]) {
	const match = NODE_EFFECT.exec(effects[0] ?? "");
	return match ? { name: match[1], value: match[2] } : { name: effects[0] ?? "", value: "" };
}

function BoardMap({ board }: { board: Board }) {
	const info = daevanionBoard(board.id);
	const opened = new Set(board.opened);
	return (
		<div className="flex flex-col gap-1 rounded-md bg-muted/40 p-1.5">
			<span className="flex items-center justify-between gap-1">
				<span className="truncate font-semibold">{info?.name ?? `#${board.id}`}</span>
				<span className="shrink-0 text-muted-foreground tabular-nums">{info ? `${board.nodes}/${info.total}` : board.nodes}</span>
			</span>
			<div className="grid aspect-square grid-cols-15 grid-rows-15 gap-px">
				{daevanionNodes(board.id).map((node) => (
					<span
						key={node.id}
						title={nodeEffects(node).join(" · ")}
						style={{ gridRow: node.row, gridColumn: node.col }}
						className={cn("flex flex-col items-center justify-center overflow-hidden text-center leading-none", node.type === "skill" ? "rounded-md" : "rounded-[2px]", opened.has(node.id) ? cn(CELLS[node.grade], "text-neutral-950") : "bg-white/5 text-white/30")}
					>
						{node.type === "start" ? null : (
							<>
								<span className="line-clamp-2 w-full px-px text-[8px] leading-[9px] font-medium break-words">{nodeLabel(nodeEffects(node)).name}</span>
								<b className="text-[10px] tabular-nums">{nodeLabel(nodeEffects(node)).value}</b>
							</>
						)}
					</span>
				))}
			</div>
		</div>
	);
}

function Daevanion({ boards }: { boards: Board[] }) {
	const t = useDictionary();
	if (boards.length === 0) {
		return null;
	}
	const opened = boards.flatMap((board) => board.opened);
	return (
		<div className="flex flex-col gap-1.5 text-xs">
			<div className="grid grid-cols-1 gap-2 2xl:grid-cols-2">
				{boards.map((board) => (
					<BoardMap key={board.id} board={board} />
				))}
			</div>
			<Line title={t.character.stats}>
				{daevanionTotals(opened, "stat").map((total) => (
					<span key={total.name}>
						<span className="text-muted-foreground">{total.name}</span> <b className="tabular-nums">+{total.value}</b>
					</span>
				))}
			</Line>
			<Line title={t.character.skills}>
				{daevanionTotals(opened, "skill").map((total) => (
					<span key={total.name}>
						<span className="text-muted-foreground">{total.name}</span> <b className="tabular-nums">+{total.value}</b>
					</span>
				))}
			</Line>
		</div>
	);
}

const SLOT_COLORS = ["bg-neutral-400", "bg-emerald-400", "bg-sky-400", "bg-amber-400", "bg-fuchsia-400"];
const SLOTS = 9;
const SPECIES_WORDS = /\b(Intellia|Bestia|Natura|Varius|Singulia|Cogni|Fera|Varian|Special)\b/g;
const EXPERIENCE_STEP = 10_000;

function shortStat(name: string) {
	return name.replace(SPECIES_WORDS, "").replace(/\s+/g, " ").trim();
}

export function Perception({ species }: { species: Species[] }) {
	const t = useDictionary();
	if (species.length === 0) {
		return null;
	}
	return (
		<div className="grid gap-2 text-xs md:grid-cols-2">
			{species.map((entry) => {
				const totals = new Map<number, number>();
				for (const effect of entry.effects) {
					totals.set(effect.stat, (totals.get(effect.stat) ?? 0) + effect.value);
				}
				return (
					<div key={entry.id} className="flex flex-col gap-2 rounded-md bg-muted/40 p-2">
						<span className="flex items-center gap-2">
							<span className="text-sm font-semibold">{speciesName(entry.id)}</span>
							<span className="rounded bg-violet-500/20 px-1.5 text-violet-300 tabular-nums">{t.detail.level(entry.level)}</span>
							<span className="ml-auto text-[10px] text-muted-foreground tabular-nums">
								{entry.experience.toLocaleString(language())} / {EXPERIENCE_STEP.toLocaleString(language())}
							</span>
						</span>
						<span className="h-1.5 w-full overflow-hidden rounded-full bg-sky-400/15">
							<span className="block h-full rounded-full bg-sky-400" style={{ width: `${Math.min(100, (entry.experience / EXPERIENCE_STEP) * 100)}%` }} />
						</span>
						<span className="flex flex-wrap gap-1">
							{Array.from({ length: SLOTS }, (_, index) => {
								const effect = entry.effects[index];
								return (
									<span
										key={index}
										title={effect ? `${statName(effect.stat)} ${statValue(effect.stat, effect.value)}` : t.character.locked(index + 1)}
										className={cn("flex size-10 flex-col items-center justify-center rounded-full leading-none", effect ? cn(SLOT_COLORS[effect.grade - 1] ?? SLOT_COLORS[0], "text-neutral-950") : "border border-dashed border-white/20 text-[10px] text-white/30")}
									>
										{effect ? (
											<>
												<span className="line-clamp-2 w-9 text-center text-[7px] leading-[8px] font-medium break-words">{shortStat(statName(effect.stat))}</span>
												<b className="text-[11px] tabular-nums">{statValue(effect.stat, effect.value)}</b>
											</>
										) : (
											index + 1
										)}
									</span>
								);
							})}
						</span>
						<span className="flex flex-col gap-0.5">
							{[...totals].map(([stat, value]) => (
								<span key={stat} className="flex justify-between gap-2">
									<span className="truncate text-muted-foreground">{statName(stat)}</span>
									<b className="tabular-nums">+{statValue(stat, value)}</b>
								</span>
							))}
						</span>
					</div>
				);
			})}
		</div>
	);
}

function Skills({ levels, gameClass }: { levels: SkillLevel[]; gameClass: number }) {
	const t = useDictionary();
	if (levels.length === 0) {
		return null;
	}
	const sorted = levels.filter((skill) => skillKnown(skill.id)).sort((a, b) => b.level - a.level || skillName(a.id).localeCompare(skillName(b.id)));
	return (
		<div className="flex flex-col gap-1.5 text-xs">
			<span className="flex flex-col gap-1">
				{sorted.map((skill) => (
					<span key={skill.id} className="flex h-8 items-center gap-2 rounded-md bg-muted/40 px-2">
						<SkillIcon id={skill.id} gameClass={gameClass} />
						<span className="min-w-0 flex-1 truncate">{skillName(skill.id)}</span>
						<span className="shrink-0 text-muted-foreground tabular-nums">{t.detail.level(skill.level)}</span>
					</span>
				))}
			</span>
		</div>
	);
}

export function Character() {
	const t = useDictionary();
	const [profile, setProfile] = useState<Profile | null>(null);

	useEffect(() => {
		const listener = listen<Profile | null>("profile", (event) => setProfile(event.payload));
		invoke<Profile | null>("profile").then(setProfile);
		return () => {
			listener.then((unlisten) => unlisten());
		};
	}, []);

	const sorted = [...(profile?.equipment ?? [])].sort((a, b) => itemOrder(a.id) - itemOrder(b.id));

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<span className="font-semibold">{t.character.title}</span>
				{profile && (
					<span className="ml-auto flex min-w-0 items-center gap-1.5 text-muted-foreground">
						<ClassIcon gameClass={profile.class} className="size-4" />
						<span className="truncate">{profile.name}</span>
					</span>
				)}
			</header>
			{sorted.length === 0 ? (
				<p className="m-auto max-w-sm p-4 text-center text-sm text-muted-foreground">{t.character.empty}</p>
			) : (
				<Tabs defaultValue="equipment" className="flex min-h-0 flex-1 flex-col gap-0">
					<TabsList className="mx-3 mt-2 w-auto shrink-0 self-start">
						<TabsTrigger value="equipment">{t.detail.equipment}</TabsTrigger>
						<TabsTrigger value="skills">{t.character.skills}</TabsTrigger>
						<TabsTrigger value="daevanion">{t.detail.daevanion}</TabsTrigger>
					</TabsList>
					<TabsContent value="equipment" className="min-h-0 flex-1 overflow-y-auto p-3 text-xs">
						<div className="grid items-start gap-1.5 lg:grid-cols-2">
							{sorted.map((gear, index) => (
								<GearDetail key={index} gear={gear} />
							))}
							<div className="lg:col-span-2">
								<Sets equipment={sorted} />
							</div>
						</div>
					</TabsContent>
					<TabsContent value="skills" className="min-h-0 flex-1 overflow-y-auto p-3 text-xs">
						<Skills levels={profile?.skillLevels ?? []} gameClass={profile?.class ?? 0} />
					</TabsContent>
					<TabsContent value="daevanion" className="min-h-0 flex-1 overflow-y-auto p-3 text-xs">
						<Daevanion boards={profile?.daevanion ?? []} />
					</TabsContent>
				</Tabs>
			)}
		</div>
	);
}