import { useState } from "react";
import { ClassIcon } from "@/components/class-icon";
import { skillIcon } from "@/lib/game-data";

export function SkillIcon({ id, gameClass }: { id: number; gameClass: number }) {
	const [failed, setFailed] = useState(false);
	const icon = skillIcon(id);
	if (!icon || failed) {
		return <ClassIcon gameClass={gameClass} className="relative size-5" />;
	}
	return <img src={icon} alt="" loading="lazy" onError={() => setFailed(true)} className="relative size-5 shrink-0 rounded-sm" />;
}