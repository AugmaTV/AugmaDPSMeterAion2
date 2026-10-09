import { useSyncExternalStore } from "react";

export const LANGUAGES = ["fr", "en"] as const;

export type Language = (typeof LANGUAGES)[number];

const KEY = "language";

const fr = {
	language: "Langue",
	tabs: { damage: "Dégâts", healing: "Soins", taken: "Subis", deaths: "Morts" },
	classes: ["Inconnu", "Gladiateur", "Templier", "Assassin", "Rôdeur", "Sorcier", "Élémentaliste", "Clerc", "Aède", "Brawler"],
	me: "MOI",
	deathCount: (count: number) => `${count} morts`,
	footer: "AION 2, noms et icônes © NCSOFT Corporation — projet non affilié à NCSOFT",
	header: {
		allPlayers: "Afficher tous les joueurs",
		partyOnly: "Afficher seulement moi et mon groupe",
		currentFight: "Afficher seulement le combat en cours",
		dungeon: "Cumuler tous les combats du donjon",
		lists: "Revenir aux listes",
		chart: "Courbe du combat",
		history: "Historique des sessions",
		character: "Mon personnage",
		unlockOverlays: "Déverrouiller les overlays (Ctrl+Shift+L)",
		lockOverlays: "Verrouiller les overlays (Ctrl+Shift+L)",
		openOverlay: "Ouvrir un overlay sur l'onglet actif",
		openOverlayOf: (tab: string) => `Ouvrir un overlay ${tab}`,
		reset: "Réinitialiser",
		copy: "Copier le résumé pour Discord",
		copied: "Résumé copié",
	},
	status: {
		unavailable: "Impossible de lire le trafic réseau. Relance le meter et accepte la demande administrateur.",
		waitingFight: "En attente d'un combat…",
		waitingGame: "En attente du jeu…",
		empty: { damage: "Aucun dégât", healing: "Aucun soin", taken: "Aucun dégât subi" },
	},
	deaths: {
		empty: "Aucune mort",
		title: "Morts",
		revivedTitle: "Ressuscité par un Clerc",
		revived: (count: number) => `${count} rés.`,
		resurrections: "Résurrections effectuées",
		deathAt: (time: string) => `Mort à ${time}`,
		noBlows: "Aucun coup reçu enregistré",
		unknownMonster: "Monstre inconnu",
	},
	detail: {
		total: "Total",
		activeDps: "DPS actif",
		active: "Actif",
		critical: "Critique",
		perfect: "Perfect",
		hard: "Puissant",
		back: "Dos",
		front: "Face",
		additional: "Additionnels",
		heals: "Soins",
		useful: "Utiles",
		surplus: "Surplus",
		net: "Net",
		gross: "Brut",
		absorbed: "Absorbé",
		absorbedTitle: (gross: string, absorbed: string) => `Brut ${gross} · absorbé ${absorbed}`,
		takenHits: "Coups reçus",
		evasions: "Esquives",
		resists: "Résistances",
		parries: "Parades",
		blocks: "Blocages",
		perfectBlocks: "Perfect Block",
		ironWalls: "Mur de fer",
		unknown: "Inconnu",
		castCount: (count: number) => `${count} lanc.`,
		hitCount: (count: number) => `${count} coups`,
		averageHeal: "Moyenne par soin",
		average: "Moyenne",
		max: "Coup max",
		casts: "Lancements",
		perMinute: "Par minute",
		specializations: "Spécialisations",
		specialization: (slot: string) => `Spé ${slot}`,
		tier: (tier: number) => `Palier ${tier}`,
		rates: { damage: "DPS", healing: "Soins/s", taken: "Subis/s" },
		compareWith: "Comparer avec",
		nobody: "Personne",
		buffs: "Buffs reçus",
		equipment: "Équipement",
		itemLevel: (level: number) => `Niveau ${level}`,
		bound: "Lien d'âme",
		mainStats: "Base",
		fixedStats: "Bonus",
		exceed: "Dépassement",
		arcanaSkills: "Compétences",
		sets: "Sets",
		pieces: (count: number) => `${count} pièce${count > 1 ? "s" : ""}`,
		daevanion: "Daevanion",
		level: (level: number) => `Niv. ${level}`,
		debuffs: "Debuffs sur la cible",
	},
	boss: "Cible",
	character: {
		title: "Mon personnage",
		empty: "Ton équipement complet s'affichera à ta prochaine connexion (sélection de personnage, puis entrée dans le monde) avec le meter lancé. Il sera ensuite gardé d'un lancement à l'autre.",
		skills: "Compétences",
		perception: "Perception d'espèce",
		stats: "Stats",
		experience: (value: string) => `${value} exp`,
		total: "Total",
	},
	sessions: {
		openWorld: "Monde ouvert",
		backToMeter: "Retour au meter",
		title: "Historique",
		count: (count: number) => `${count} sessions`,
		empty: "Aucune session enregistrée",
		selectAll: "Tout sélectionner",
		selected: (count: number) => `${count} sélectionnée${count > 1 ? "s" : ""}`,
		compareTitle: "Comparer les deux sessions",
		compareHint: "Coche exactement deux sessions",
		compare: "Comparer",
		fights: (count: number) => `${count} ${count > 1 ? "combats" : "combat"}`,
		locked: "Session protégée : cliquer pour la déverrouiller",
		lock: "Protéger de la suppression automatique",
		rename: "Renommer",
		confirmDelete: "Cliquer à nouveau pour supprimer",
		delete: "Supprimer",
		backToHistory: "Retour à l'historique",
		fight: (number: number) => `Combat ${number}`,
		whole: "Session complète",
		comparison: "Comparaison",
		gap: "Écart",
		date: "Date",
		duration: "Durée",
		groupDps: "DPS du groupe",
	},
	timeline: {
		notEnough: "Pas encore assez de données pour tracer la courbe",
		others: (count: number) => `Autres (${count})`,
		chart: "Graphique",
		table: "Tableau",
		time: "Temps",
		bossHealth: "Vie du boss",
		dps: (seconds: number) => `DPS (moyenne sur ${seconds} s)`,
	},
	overlay: {
		lock: "Verrouiller (Ctrl+Shift+L)",
		close: "Fermer",
	},
	update: {
		checking: "Recherche de mises à jour…",
		downloading: (version: string, progress: number | null) => `Téléchargement de la v${version}${progress === null ? "…" : ` · ${progress}%`}`,
		installing: (version: string) => `Installation de la v${version}, le meter va redémarrer…`,
	},
	summary: {
		fight: "Combat",
	},
	skills: {
		drain: "Vol de vie",
		basicAttack: "Attaque de base",
	},
};

export type Dictionary = typeof fr;

const en: Dictionary = {
	language: "Language",
	tabs: { damage: "Damage", healing: "Healing", taken: "Taken", deaths: "Deaths" },
	classes: ["Unknown", "Gladiator", "Templar", "Assassin", "Ranger", "Sorcerer", "Elementalist", "Cleric", "Chanter", "Brawler"],
	me: "ME",
	deathCount: (count: number) => `${count} deaths`,
	footer: "AION 2, names and icons © NCSOFT Corporation — not affiliated with NCSOFT",
	header: {
		allPlayers: "Show all players",
		partyOnly: "Show only me and my party",
		currentFight: "Show only the current fight",
		dungeon: "Add up every fight of the dungeon",
		lists: "Back to the lists",
		chart: "Fight timeline",
		history: "Session history",
		character: "My character",
		unlockOverlays: "Unlock overlays (Ctrl+Shift+L)",
		lockOverlays: "Lock overlays (Ctrl+Shift+L)",
		openOverlay: "Open an overlay on the active tab",
		openOverlayOf: (tab: string) => `Open a ${tab} overlay`,
		reset: "Reset",
		copy: "Copy the summary for Discord",
		copied: "Summary copied",
	},
	status: {
		unavailable: "Cannot read network traffic. Restart the meter and accept the administrator prompt.",
		waitingFight: "Waiting for a fight…",
		waitingGame: "Waiting for the game…",
		empty: { damage: "No damage", healing: "No healing", taken: "No damage taken" },
	},
	deaths: {
		empty: "No deaths",
		title: "Deaths",
		revivedTitle: "Resurrected by a Cleric",
		revived: (count: number) => `${count} res.`,
		resurrections: "Resurrections cast",
		deathAt: (time: string) => `Died at ${time}`,
		noBlows: "No hits recorded",
		unknownMonster: "Unknown monster",
	},
	detail: {
		total: "Total",
		activeDps: "Active DPS",
		active: "Active",
		critical: "Critical",
		perfect: "Perfect",
		hard: "Hard hit",
		back: "Back",
		front: "Front",
		additional: "Additional",
		heals: "Heals",
		useful: "Useful",
		surplus: "Overheal",
		net: "Net",
		gross: "Gross",
		absorbed: "Absorbed",
		absorbedTitle: (gross: string, absorbed: string) => `Gross ${gross} · absorbed ${absorbed}`,
		takenHits: "Hits taken",
		evasions: "Evasions",
		resists: "Resists",
		parries: "Parries",
		blocks: "Blocks",
		perfectBlocks: "Perfect Block",
		ironWalls: "Iron Wall",
		unknown: "Unknown",
		castCount: (count: number) => `${count} casts`,
		hitCount: (count: number) => `${count} hits`,
		averageHeal: "Average heal",
		average: "Average",
		max: "Max hit",
		casts: "Casts",
		perMinute: "Per minute",
		specializations: "Specializations",
		specialization: (slot: string) => `Spec ${slot}`,
		tier: (tier: number) => `Tier ${tier}`,
		rates: { damage: "DPS", healing: "Heal/s", taken: "Taken/s" },
		compareWith: "Compare with",
		nobody: "Nobody",
		buffs: "Buffs received",
		equipment: "Equipment",
		itemLevel: (level: number) => `Level ${level}`,
		bound: "Soul bind",
		mainStats: "Base",
		fixedStats: "Bonus",
		exceed: "Exceed",
		arcanaSkills: "Skills",
		sets: "Sets",
		pieces: (count: number) => `${count} piece${count > 1 ? "s" : ""}`,
		daevanion: "Daevanion",
		level: (level: number) => `Lv. ${level}`,
		debuffs: "Debuffs on target",
	},
	boss: "Target",
	character: {
		title: "My character",
		empty: "Your full equipment will show up at your next login (character select, then entering the world) while the meter is running. It is then kept between launches.",
		skills: "Skills",
		perception: "Species perception",
		stats: "Stats",
		experience: (value: string) => `${value} exp`,
		total: "Total",
	},
	sessions: {
		openWorld: "Open world",
		backToMeter: "Back to the meter",
		title: "History",
		count: (count: number) => `${count} sessions`,
		empty: "No saved session",
		selectAll: "Select all",
		selected: (count: number) => `${count} selected`,
		compareTitle: "Compare the two sessions",
		compareHint: "Tick exactly two sessions",
		compare: "Compare",
		fights: (count: number) => `${count} ${count > 1 ? "fights" : "fight"}`,
		locked: "Protected session: click to unlock",
		lock: "Protect from automatic deletion",
		rename: "Rename",
		confirmDelete: "Click again to delete",
		delete: "Delete",
		backToHistory: "Back to history",
		fight: (number: number) => `Fight ${number}`,
		whole: "Whole session",
		comparison: "Comparison",
		gap: "Change",
		date: "Date",
		duration: "Duration",
		groupDps: "Party DPS",
	},
	timeline: {
		notEnough: "Not enough data yet to draw the timeline",
		others: (count: number) => `Others (${count})`,
		chart: "Chart",
		table: "Table",
		time: "Time",
		bossHealth: "Boss health",
		dps: (seconds: number) => `DPS (${seconds} s average)`,
	},
	overlay: {
		lock: "Lock (Ctrl+Shift+L)",
		close: "Close",
	},
	update: {
		checking: "Checking for updates…",
		downloading: (version: string, progress: number | null) => `Downloading v${version}${progress === null ? "…" : ` · ${progress}%`}`,
		installing: (version: string) => `Installing v${version}, the meter will restart…`,
	},
	summary: {
		fight: "Fight",
	},
	skills: {
		drain: "Life drain",
		basicAttack: "Basic attack",
	},
};

const DICTIONARIES: Record<Language, Dictionary> = { fr, en };

const listeners = new Set<() => void>();

let current = stored();

window.addEventListener("storage", (event) => {
	if (event.key === KEY) {
		current = stored();
		notify();
	}
});

export function language() {
	return current;
}

export function setLanguage(next: Language) {
	localStorage.setItem(KEY, next);
	current = next;
	notify();
}

export function dictionary() {
	return DICTIONARIES[current];
}

export function useDictionary() {
	return DICTIONARIES[useSyncExternalStore(subscribe, language)];
}

function stored(): Language {
	const saved = localStorage.getItem(KEY);
	if (saved === "fr" || saved === "en") {
		return saved;
	}
	return navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
}

function notify() {
	listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void) {
	listeners.add(listener);
	return () => {
		listeners.delete(listener);
	};
}