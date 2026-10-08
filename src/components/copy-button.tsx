import { useEffect, useState } from "react";
import { Check, ClipboardCopy } from "lucide-react";
import { Button } from "@/components/ui/button";

export function CopyButton({ text }: { text: () => string | null }) {
	const [copied, setCopied] = useState(false);

	useEffect(() => {
		if (!copied) {
			return;
		}
		const timer = setTimeout(() => setCopied(false), 1500);
		return () => clearTimeout(timer);
	}, [copied]);

	const copy = () => {
		const value = text();
		if (value) {
			navigator.clipboard.writeText(value).then(() => setCopied(true));
		}
	};

	return (
		<Button variant="ghost" size="icon-sm" title={copied ? "Résumé copié" : "Copier le résumé pour Discord"} onClick={copy}>
			{copied ? <Check /> : <ClipboardCopy />}
		</Button>
	);
}