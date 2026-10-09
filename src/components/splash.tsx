import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import icon from "@/assets/icon.png";
import { useDictionary, type Dictionary } from "@/lib/i18n";
import { cn } from "@/lib/utils";

type UpdateState = { stage: "checking" } | { stage: "downloading"; version: string; progress: number | null } | { stage: "installing"; version: string };

function describe(state: UpdateState, t: Dictionary) {
	switch (state.stage) {
		case "checking":
			return t.update.checking;
		case "downloading":
			return t.update.downloading(state.version, state.progress);
		case "installing":
			return t.update.installing(state.version);
	}
}

export function Splash() {
	const t = useDictionary();
	const [state, setState] = useState<UpdateState>({ stage: "checking" });

	useEffect(() => {
		const listener = listen<UpdateState>("update", (event) => setState(event.payload));
		return () => {
			listener.then((unlisten) => unlisten());
		};
	}, []);

	const progress = state.stage === "downloading" ? state.progress : state.stage === "installing" ? 100 : null;

	return (
		<main data-tauri-drag-region className="flex h-screen items-center gap-4 border bg-background px-5 text-foreground select-none">
			<img src={icon} alt="" className="size-16 shrink-0" />
			<div data-tauri-drag-region className="flex min-w-0 flex-1 flex-col gap-2">
				<span data-tauri-drag-region className="text-base font-semibold">Augma DPS Meter</span>
				<span data-tauri-drag-region className="text-xs text-muted-foreground">{describe(state, t)}</span>
				<div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
					<div className={cn("h-full rounded-full bg-violet-400 transition-[width]", progress === null && "w-1/3 animate-pulse")} style={progress === null ? undefined : { width: `${progress}%` }} />
				</div>
			</div>
		</main>
	);
}