import { useEffect, useState } from "react";
import { Check, ClipboardCopy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useDictionary } from "@/lib/i18n";

export function CopyButton({ text }: { text: () => string | null }) {
	const t = useDictionary();
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
		<Button variant="ghost" size="icon-sm" title={copied ? t.header.copied : t.header.copy} onClick={copy}>
			{copied ? <Check /> : <ClipboardCopy />}
		</Button>
	);
}