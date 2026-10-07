import { useEffect, useState } from "react";
import { PictureInPicture2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { MeterView } from "@/components/meter-view";
import { compact } from "@/lib/format";
import { MODES, TITLES, totalRate, type Mode, type Snapshot } from "@/lib/meter";

export function Board({ snapshot, mode, onMode, onOverlay }: { snapshot: Snapshot | null; mode: Mode; onMode: (mode: Mode) => void; onOverlay?: (mode: Mode) => void }) {
	const [selected, setSelected] = useState<{ id: number; mode: Mode } | null>(null);

	useEffect(() => {
		setSelected((selected) => (snapshot?.players.some((player) => player.id === selected?.id) ? selected : null));
	}, [snapshot]);

	return (
		<>
			<Tabs value={mode} onValueChange={(value) => onMode(value as Mode)} className="shrink-0 px-2 pt-2 lg:hidden">
				<TabsList className="w-full">
					{MODES.map((tab) => (
						<TabsTrigger key={tab} value={tab}>
							{TITLES[tab]}
						</TabsTrigger>
					))}
				</TabsList>
			</Tabs>
			<div className="flex min-h-0 flex-1 flex-col lg:hidden">
				<MeterView snapshot={snapshot} mode={mode} selected={selected?.id ?? null} onSelect={(id) => setSelected(id === null ? null : { id, mode })} />
			</div>
			<div className="hidden min-h-0 flex-1 divide-x lg:flex">
				{MODES.map((column) => (
					<div key={column} className="flex min-w-0 flex-1 flex-col">
						<div className="flex h-9 shrink-0 items-center gap-2 border-b px-3 text-sm">
							<span className="font-semibold">{TITLES[column]}</span>
							<span className="ml-auto font-semibold tabular-nums">{compact(totalRate(snapshot?.players ?? [], column))}/s</span>
							{onOverlay && (
								<Button variant="ghost" size="icon-xs" title={`Ouvrir un overlay ${TITLES[column]}`} onClick={() => onOverlay(column)}>
									<PictureInPicture2 />
								</Button>
							)}
						</div>
						<MeterView snapshot={snapshot} mode={column} selected={selected?.mode === column ? selected.id : null} onSelect={(id) => setSelected(id === null ? null : { id, mode: column })} />
					</div>
				))}
			</div>
		</>
	);
}