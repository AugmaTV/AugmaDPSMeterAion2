import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Check, X } from "lucide-react";
import { useDictionary } from "@/lib/i18n";
import type { Diagnostic } from "@/lib/meter";

const SOURCES = ["character", "equipment", "skills", "daevanion", "pets"] as const;

export function SyncStatus() {
	const t = useDictionary();
	const [diagnostics, setDiagnostics] = useState<Record<string, Diagnostic>>({});

	useEffect(() => {
		const listener = listen<Record<string, Diagnostic>>("diagnostics", (event) => setDiagnostics(event.payload));
		invoke<Record<string, Diagnostic>>("diagnostics").then(setDiagnostics);
		return () => {
			listener.then((unlisten) => unlisten());
		};
	}, []);

	const received = SOURCES.some((source) => diagnostics[source]);

	return (
		<div className="flex w-full max-w-sm flex-col gap-1.5 text-xs">
			<span className="text-[10px] font-semibold text-muted-foreground">{t.sync.title}</span>
			{SOURCES.map((source) => {
				const entry = diagnostics[source];
				return (
					<span key={source} className="flex items-center gap-2 rounded-md bg-muted/40 px-2 py-1">
						{entry?.decoded ? <Check className="size-3.5 text-emerald-400" /> : <X className="size-3.5 text-muted-foreground" />}
						<span className="flex-1">{t.sync.sources[source]}</span>
						<span className="text-muted-foreground tabular-nums">{entry ? `${entry.decoded ? entry.items : t.sync.unreadable} · ${entry.size} o · ×${entry.count}` : t.sync.none}</span>
					</span>
				);
			})}
			{!received && <p className="text-muted-foreground">{t.sync.hint}</p>}
		</div>
	);
}
