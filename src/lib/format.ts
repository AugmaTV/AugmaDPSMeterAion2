export function compact(value: number) {
	if (value >= 1_000_000) {
		return `${(value / 1_000_000).toFixed(2)}M`;
	}
	if (value >= 1_000) {
		return `${(value / 1_000).toFixed(1)}k`;
	}
	return `${Math.round(value)}`;
}

export function clock(millis: number) {
	const seconds = Math.floor(millis / 1000);
	return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

export function percent(part: number, total: number) {
	return `${total ? Math.round((part / total) * 100) : 0}%`;
}