import { useState } from "react";
import { petIcon } from "@/lib/game-data";
import { cn } from "@/lib/utils";

export function PetIcon({ id, className }: { id: number; className?: string }) {
	const [failed, setFailed] = useState(false);
	const icon = petIcon(id);
	if (!icon || failed) {
		return <span className={cn("shrink-0 rounded-sm bg-muted", className)} />;
	}
	return <img src={icon} alt="" loading="lazy" onError={() => setFailed(true)} className={cn("shrink-0 rounded-sm", className)} />;
}