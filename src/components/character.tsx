import { useEffect, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ChevronLeft, Gem } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ClassIcon } from "@/components/class-icon";
import { GearIcon, gradeBorder, gradeText } from "@/components/gear-icon";
import { SkillIcon } from "@/components/skill-icon";
import { daevanionBoard, daevanionNodes, daevanionTotals, detailName, godstoneName, gradeName, itemDetail, itemGrade, itemName, itemOrder, itemSet, localized, localizedValue, nodeEffects, skillKnown, skillName, speciesName, statId, statName, statValue, stoneValue } from "@/lib/game-data";
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
						className={cn(node.type === "skill" ? "rounded-full" : "rounded-[1px]", opened.has(node.id) ? CELLS[node.grade] : "bg-white/10")}
					/>
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
			<span className="font-semibold text-muted-foreground">{t.detail.daevanion}</span>
			<div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
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

function Perception({ species }: { species: Species[] }) {
	const t = useDictionary();
	if (species.length === 0) {
		return null;
	}
	return (
		<div className="flex flex-col gap-1.5 text-xs">
			<span className="font-semibold text-muted-foreground">{t.character.perception}</span>
			{species.map((entry) => {
				const totals = new Map<number, number>();
				for (const effect of entry.effects) {
					totals.set(effect.stat, (totals.get(effect.stat) ?? 0) + effect.value);
				}
				return (
					<div key={entry.id} className="flex flex-col gap-1 rounded-md bg-muted/40 p-1.5">
						<span className="flex items-center gap-2">
							<span className="font-semibold">{speciesName(entry.id)}</span>
							<span className="text-muted-foreground tabular-nums">{t.detail.level(entry.level)}</span>
							<span className="ml-auto text-[10px] text-muted-foreground tabular-nums">{t.character.experience(entry.experience.toLocaleString(language()))}</span>
						</span>
						<span className="flex flex-wrap gap-1">
							{entry.effects.map((effect, position) => (
								<span key={position} className={cn("rounded border px-1 text-[10px]", gradeBorder(gradeName(effect.grade)), gradeText(gradeName(effect.grade)))}>
									{statName(effect.stat)} <b className="tabular-nums">{statValue(effect.stat, effect.value)}</b>
								</span>
							))}
						</span>
						<Line title={t.character.total}>
							{[...totals].map(([stat, value]) => (
								<span key={stat}>
									<span className="text-muted-foreground">{statName(stat)}</span> <b className="tabular-nums">{statValue(stat, value)}</b>
								</span>
							))}
						</Line>
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
			<span className="font-semibold text-muted-foreground">{t.character.skills}</span>
			<span className="grid grid-cols-1 gap-1 sm:grid-cols-2">
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

export function Character({ onBack }: { onBack: () => void }) {
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
		<main className="flex h-screen flex-col bg-background text-foreground select-none">
			<header className="flex h-11 shrink-0 items-center gap-2 border-b px-3 text-sm">
				<Button variant="ghost" size="icon-sm" title={t.sessions.backToMeter} onClick={onBack}>
					<ChevronLeft />
				</Button>
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
				<section className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-3 text-xs">
					<div className="flex flex-col gap-1">
						<span className="font-semibold text-muted-foreground">{t.detail.equipment}</span>
						{sorted.map((gear, index) => (
							<GearDetail key={index} gear={gear} />
						))}
						<Sets equipment={sorted} />
					</div>
					<Daevanion boards={profile?.daevanion ?? []} />
					<Perception species={profile?.perception ?? []} />
					<Skills levels={profile?.skillLevels ?? []} gameClass={profile?.class ?? 0} />
				</section>
			)}
		</main>
	);
}