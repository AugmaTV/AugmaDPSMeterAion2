import { useState } from "react";
import { itemGrade, itemIcon, itemName } from "@/lib/game-data";
import type { Gear } from "@/lib/meter";
import { cn } from "@/lib/utils";

const GRADES: Record<string, { border: string; text: string }> = {
	Common: { border: "border-neutral-500", text: "text-neutral-300" },
	Rare: { border: "border-emerald-400", text: "text-emerald-400" },
	Legend: { border: "border-sky-400", text: "text-sky-400" },
	Unique: { border: "border-amber-400", text: "text-amber-400" },
	Special: { border: "border-fuchsia-400", text: "text-fuchsia-400" },
	Epic: { border: "border-violet-400", text: "text-violet-400" },
};

export function gradeText(grade: string | null) {
	return GRADES[grade ?? ""]?.text ?? "text-foreground";
}

export function gradeBorder(grade: string | null) {
	return GRADES[grade ?? ""]?.border ?? "border-border";
}

export function GearIcon({ gear }: { gear: Gear }) {
	const [failed, setFailed] = useState(false);
	const icon = itemIcon(gear.id);
	return (
		<span className={cn("relative size-8 shrink-0 overflow-hidden rounded-md border bg-background", gradeBorder(itemGrade(gear.id)))} title={`${itemName(gear.id)} +${gear.enchant}`}>
			{icon && !failed && <img src={icon} alt="" loading="lazy" onError={() => setFailed(true)} className="size-full" />}
			{gear.enchant > 0 && <span className="absolute right-0 bottom-0 rounded-tl bg-black/75 px-0.5 text-[9px] leading-tight font-bold tabular-nums">+{gear.enchant}</span>}
		</span>
	);
}