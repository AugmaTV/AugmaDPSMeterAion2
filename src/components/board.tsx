import { useEffect, useState } from "react";
import { PictureInPicture2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DeathsView } from "@/components/deaths-view";
import { MeterView } from "@/components/meter-view";
import { compact } from "@/lib/format";
import { useDictionary } from "@/lib/i18n";
import { TABS, totalRate, type Mode, type Player, type Snapshot, type Tab } from "@/lib/meter";

export function Headline({ players, tab }: { players: Player[]; tab: Tab }) {
	const t = useDictionary();
	return tab === "deaths" ? t.deathCount(players.reduce((sum, player) => sum + player.deaths, 0)) : `${compact(totalRate(players, tab))}/s`;
}

export function Board({ snapshot, mode, onMode, onOverlay }: { snapshot: Snapshot | null; mode: Tab; onMode: (mode: Tab) => void; onOverlay?: (mode: Mode) => void }) {
	const t = useDictionary();
	const [selected, setSelected] = useState<{ id: number; mode: Mode } | null>(null);

	useEffect(() => {
		setSelected((selected) => (snapshot?.players.some((player) => player.id === selected?.id) ? selected : null));
	}, [snapshot]);

	return (
		<>
			<Tabs value={mode} onValueChange={(value) => onMode(value as Tab)} className="shrink-0 px-2 pt-2 xl:hidden">
				<TabsList className="w-full">
					{TABS.map((tab) => (
						<TabsTrigger key={tab} value={tab}>
							{t.tabs[tab]}
						</TabsTrigger>
					))}
				</TabsList>
			</Tabs>
			<div className="flex min-h-0 flex-1 flex-col xl:hidden">
				{mode === "deaths" ? <DeathsView snapshot={snapshot} /> : <MeterView snapshot={snapshot} mode={mode} selected={selected?.id ?? null} onSelect={(id) => setSelected(id === null ? null : { id, mode })} />}
			</div>
			<div className="hidden min-h-0 flex-1 divide-x xl:flex">
				{TABS.map((column) => (
					<div key={column} className="flex min-w-0 flex-1 flex-col">
						<div className="flex h-9 shrink-0 items-center gap-2 border-b px-3 text-sm">
							<span className="font-semibold">{t.tabs[column]}</span>
							<span className="ml-auto font-semibold tabular-nums">
								<Headline players={snapshot?.players ?? []} tab={column} />
							</span>
							{onOverlay && column !== "deaths" && (
								<Button variant="ghost" size="icon-xs" title={t.header.openOverlayOf(t.tabs[column])} onClick={() => onOverlay(column)}>
									<PictureInPicture2 />
								</Button>
							)}
						</div>
						{column === "deaths" ? <DeathsView snapshot={snapshot} /> : <MeterView snapshot={snapshot} mode={column} selected={selected?.mode === column ? selected.id : null} onSelect={(id) => setSelected(id === null ? null : { id, mode: column })} />}
					</div>
				))}
			</div>
		</>
	);
}