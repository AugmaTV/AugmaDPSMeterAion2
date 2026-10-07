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
	name: string;
	color: string;
	icon: string | null;
};

export const CLASSES: GameClass[] = [
	{ name: "Inconnu", color: "#737373", icon: null },
	{ name: "Gladiateur", color: "#e0893a", icon: gladiator },
	{ name: "Templier", color: "#5b8def", icon: templar },
	{ name: "Assassin", color: "#b46ae0", icon: assassin },
	{ name: "Rôdeur", color: "#7cc04f", icon: ranger },
	{ name: "Sorcier", color: "#4fb7e0", icon: sorcerer },
	{ name: "Élémentaliste", color: "#8f7cf0", icon: elementalist },
	{ name: "Clerc", color: "#e8c94a", icon: cleric },
	{ name: "Aède", color: "#3fbf9f", icon: chanter },
	{ name: "Brawler", color: "#e05a5a", icon: brawler },
];