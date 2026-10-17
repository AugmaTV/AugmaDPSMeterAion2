import { ClassIcon } from "@/components/class-icon";
import { selectCharacter, useCharacters } from "@/lib/characters";

export function CharacterPicker() {
	const { characters, selected } = useCharacters();
	if (!selected) {
		return null;
	}
	return (
		<span className="ml-auto flex min-w-0 items-center gap-1.5">
			<ClassIcon gameClass={selected.class} className="size-4" />
			<select value={selected.name} onChange={(event) => selectCharacter(event.target.value)} className="h-7 min-w-0 rounded border bg-background px-1.5 text-xs">
				{characters.map((profile) => (
					<option key={profile.name} value={profile.name}>
						{profile.name}
					</option>
				))}
			</select>
		</span>
	);
}
