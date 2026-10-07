import { CLASSES } from "@/lib/classes";
import { cn } from "@/lib/utils";

export function ClassIcon({ gameClass, className }: { gameClass: number; className?: string }) {
	const { color, icon } = CLASSES[gameClass];
	if (!icon) {
		return <span className={cn("size-5 shrink-0 rounded-full", className)} style={{ backgroundColor: color }} />;
	}
	return <span className={cn("size-5 shrink-0", className)} style={{ backgroundColor: color, maskImage: `url(${icon})`, maskSize: "contain", maskRepeat: "no-repeat", maskPosition: "center" }} />;
}