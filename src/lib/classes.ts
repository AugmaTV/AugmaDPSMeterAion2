import assassin from "@/assets/classes/assassin.png";
import brawler from "@/assets/classes/brawler.png";
import chanter from "@/assets/classes/chanter.png";
import cleric from "@/assets/classes/cleric.png";
import elementalist from "@/assets/classes/elementalist.png";
import gladiator from "@/assets/classes/gladiator.png";
import ranger from "@/assets/classes/ranger.png";
import sorcerer from "@/assets/classes/sorcerer.png";
import templar from "@/assets/classes/templar.png";

export type GameClass = {
	color: string;
	icon: string | null;
};

export const CLASSES: GameClass[] = [
	{ color: "#737373", icon: null },
	{ color: "#e0893a", icon: gladiator },
	{ color: "#5b8def", icon: templar },
	{ color: "#b46ae0", icon: assassin },
	{ color: "#7cc04f", icon: ranger },
	{ color: "#4fb7e0", icon: sorcerer },
	{ color: "#8f7cf0", icon: elementalist },
	{ color: "#e8c94a", icon: cleric },
	{ color: "#3fbf9f", icon: chanter },
	{ color: "#e05a5a", icon: brawler },
];