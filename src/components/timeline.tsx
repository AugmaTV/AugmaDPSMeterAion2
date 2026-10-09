import { useEffect, useRef, useState, type PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import { clock, compact } from "@/lib/format";
import { useDictionary } from "@/lib/i18n";
import type { Snapshot } from "@/lib/meter";

const SERIES = ["#3987e5", "#d95926", "#199e70", "#c98500", "#d55181", "#008300", "#9085e9", "#e66767"];
const OTHERS = "#737373";
const BOSS = "#fb7185";
const WINDOW = 5;
const TABLE_STEP = 10;
const DPS_HEIGHT = 180;
const BOSS_HEIGHT = 64;
const AXIS = 20;
const GUTTER = 44;
const PADDING = 12;
const TICK_STEPS = [5, 10, 30, 60, 120, 300, 600, 1200];
const MAX_TICKS = 6;

type Series = {
	key: string;
	name: string;
	color: string;
	values: number[];
};

function smooth(timeline: number[], length: number) {
	return Array.from({ length }, (_, second) => {
		const from = Math.max(0, second - WINDOW + 1);
		return timeline.slice(from, second + 1).reduce((sum, value) => sum + value, 0) / (second - from + 1);
	});
}

function ceiling(value: number) {
	if (value <= 0) {
		return 1;
	}
	const power = 10 ** Math.floor(Math.log10(value));
	return ([1, 2, 5, 10].find((step) => step * power >= value) ?? 10) * power;
}

function path(values: (number | null)[], x: (index: number) => number, y: (value: number) => number) {
	return values.reduce((line, value, index) => (value === null ? line : `${line}${index === 0 || values[index - 1] === null ? "M" : "L"}${x(index).toFixed(1)},${y(value).toFixed(1)}`), "");
}

export function Timeline({ snapshot }: { snapshot: Snapshot | null }) {
	const t = useDictionary();
	const container = useRef<HTMLElement>(null);
	const slots = useRef(new Map<number, number>());
	const [width, setWidth] = useState(0);
	const [hover, setHover] = useState<number | null>(null);
	const [table, setTable] = useState(false);

	useEffect(() => {
		const element = container.current;
		if (!element) {
			return;
		}
		const observer = new ResizeObserver(([entry]) => setWidth(entry.contentRect.width));
		observer.observe(element);
		return () => observer.disconnect();
	}, []);

	const players = snapshot?.players ?? [];
	const length = Math.max(snapshot?.health.length ?? 0, ...players.map((player) => player.timeline.length));
	for (const player of players) {
		if (!slots.current.has(player.id) && slots.current.size < SERIES.length) {
			slots.current.set(player.id, slots.current.size);
		}
	}
	const series: Series[] = players
		.filter((player) => slots.current.has(player.id))
		.sort((a, b) => (slots.current.get(a.id) ?? 0) - (slots.current.get(b.id) ?? 0))
		.map((player) => ({ key: String(player.id), name: player.name ?? t.classes[player.class], color: SERIES[slots.current.get(player.id) ?? 0], values: smooth(player.timeline, length) }));
	const rest = players.filter((player) => !slots.current.has(player.id));
	if (rest.length > 0) {
		const merged = Array.from({ length }, (_, second) => rest.reduce((sum, player) => sum + (player.timeline[second] ?? 0), 0));
		series.push({ key: "others", name: t.timeline.others(rest.length), color: OTHERS, values: smooth(merged, length) });
	}
	const health = Array.from({ length }, (_, second) => snapshot?.health[second] ?? null);
	const top = ceiling(Math.max(0, ...series.flatMap((entry) => entry.values)));
	const plot = Math.max(width - GUTTER - PADDING, 1);
	const x = (index: number) => GUTTER + (length > 1 ? (index / (length - 1)) * plot : 0);
	const dps = (value: number) => PADDING + (1 - value / top) * (DPS_HEIGHT - PADDING);
	const boss = (value: number) => PADDING + (1 - value / 100) * (BOSS_HEIGHT - PADDING);
	const step = TICK_STEPS.find((candidate) => length / candidate <= MAX_TICKS) ?? TICK_STEPS[TICK_STEPS.length - 1];
	const ticks = Array.from({ length: Math.floor((length - 1) / step) + 1 }, (_, index) => index * step);

	const track = (event: PointerEvent<HTMLDivElement>) => {
		const bounds = event.currentTarget.getBoundingClientRect();
		const ratio = (event.clientX - bounds.left - GUTTER) / plot;
		setHover(Math.min(length - 1, Math.max(0, Math.round(ratio * (length - 1)))));
	};

	return (
		<section ref={container} className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-3 text-xs">
			{length < 2 && <p className="m-auto p-4 text-sm text-muted-foreground">{t.timeline.notEnough}</p>}
			<div className="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1">
				{series.map((entry) => (
					<span key={entry.key} className="flex items-center gap-1.5 text-muted-foreground">
						<span className="h-0.5 w-3 rounded-full" style={{ backgroundColor: entry.color }} />
						{entry.name}
					</span>
				))}
				<Button variant="ghost" size="xs" className="ml-auto" onClick={() => setTable(!table)}>
					{table ? t.timeline.chart : t.timeline.table}
				</Button>
			</div>
			{length < 2 ? null : table ? (
				<div className="overflow-x-auto">
					<table className="w-full tabular-nums">
						<thead className="text-muted-foreground">
							<tr>
								<th className="px-1 py-1 text-left font-medium">{t.timeline.time}</th>
								{series.map((entry) => (
									<th key={entry.key} className="px-1 py-1 text-right font-medium">
										{entry.name}
									</th>
								))}
								<th className="px-1 py-1 text-right font-medium">{t.timeline.bossHealth}</th>
							</tr>
						</thead>
						<tbody>
							{Array.from({ length: Math.ceil(length / TABLE_STEP) }, (_, row) => row * TABLE_STEP).map((start) => (
								<tr key={start} className="border-t">
									<td className="px-1 py-1">{clock(start * 1000)}</td>
									{series.map((entry) => (
										<td key={entry.key} className="px-1 py-1 text-right">
											{compact(entry.values.slice(start, start + TABLE_STEP).reduce((sum, value) => sum + value, 0) / Math.min(TABLE_STEP, length - start))}/s
										</td>
									))}
									<td className="px-1 py-1 text-right">{health[Math.min(start + TABLE_STEP, length) - 1] === null ? "—" : `${Math.round(health[Math.min(start + TABLE_STEP, length) - 1] ?? 0)} %`}</td>
								</tr>
							))}
						</tbody>
					</table>
				</div>
			) : (
				<div className="relative shrink-0" onPointerMove={track} onPointerLeave={() => setHover(null)}>
					<span className="font-semibold text-muted-foreground">{t.timeline.dps(WINDOW)}</span>
					<svg width={width} height={DPS_HEIGHT + AXIS} className="block overflow-visible">
						{[0, top / 2, top].map((value) => (
							<g key={value}>
								<line x1={GUTTER} x2={GUTTER + plot} y1={dps(value)} y2={dps(value)} className="stroke-border" strokeWidth={1} />
								<text x={GUTTER - 6} y={dps(value)} textAnchor="end" dominantBaseline="middle" className="fill-muted-foreground tabular-nums">
									{compact(value)}
								</text>
							</g>
						))}
						{ticks.map((second) => (
							<text key={second} x={x(second)} y={DPS_HEIGHT + 14} textAnchor="middle" className="fill-muted-foreground tabular-nums">
								{clock(second * 1000)}
							</text>
						))}
						{series.map((entry) => (
							<path key={entry.key} d={path(entry.values, x, dps)} fill="none" stroke={entry.color} strokeWidth={2} strokeLinejoin="round" strokeLinecap="round" />
						))}
						{hover !== null && <line x1={x(hover)} x2={x(hover)} y1={PADDING} y2={DPS_HEIGHT} className="stroke-muted-foreground" strokeWidth={1} />}
					</svg>
					<span className="font-semibold text-muted-foreground">{t.timeline.bossHealth}</span>
					<svg width={width} height={BOSS_HEIGHT} className="block overflow-visible">
						{[0, 100].map((value) => (
							<g key={value}>
								<line x1={GUTTER} x2={GUTTER + plot} y1={boss(value)} y2={boss(value)} className="stroke-border" strokeWidth={1} />
								<text x={GUTTER - 6} y={boss(value)} textAnchor="end" dominantBaseline="middle" className="fill-muted-foreground tabular-nums">
									{value} %
								</text>
							</g>
						))}
						<path d={path(health, x, boss)} fill="none" stroke={BOSS} strokeWidth={2} strokeLinejoin="round" strokeLinecap="round" />
						{hover !== null && <line x1={x(hover)} x2={x(hover)} y1={PADDING} y2={BOSS_HEIGHT} className="stroke-muted-foreground" strokeWidth={1} />}
					</svg>
					{hover !== null && (
						<div className="pointer-events-none absolute top-6 z-10 flex min-w-36 flex-col gap-0.5 rounded-md border bg-popover px-2 py-1.5 shadow-md" style={x(hover) > width / 2 ? { right: width - x(hover) + 8 } : { left: x(hover) + 8 }}>
							<span className="text-muted-foreground tabular-nums">{clock(hover * 1000)}</span>
							{series.map((entry) => (
								<span key={entry.key} className="flex items-center gap-1.5">
									<span className="h-0.5 w-3 shrink-0 rounded-full" style={{ backgroundColor: entry.color }} />
									<b className="tabular-nums">{compact(entry.values[hover])}/s</b>
									<span className="truncate text-muted-foreground">{entry.name}</span>
								</span>
							))}
							{health[hover] !== null && (
								<span className="flex items-center gap-1.5">
									<span className="h-0.5 w-3 shrink-0 rounded-full" style={{ backgroundColor: BOSS }} />
									<b className="tabular-nums">{Math.round(health[hover] ?? 0)} %</b>
									<span className="text-muted-foreground">{t.timeline.bossHealth}</span>
								</span>
							)}
						</div>
					)}
				</div>
			)}
		</section>
	);
}