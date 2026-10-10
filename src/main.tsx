import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "@/App";
import { Overlay } from "@/components/overlay";
import { PetOverlay } from "@/components/pet-overlay";
import { Splash } from "@/components/splash";
import type { Mode } from "@/lib/meter";
import "@/index.css";

const [view, mode] = getCurrentWebviewWindow().label.split("-");

const VIEWS: Record<string, React.ReactNode> = {
	overlay: mode === "pets" ? <PetOverlay /> : <Overlay mode={mode as Mode} />,
	splash: <Splash />,
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<React.StrictMode>{VIEWS[view] ?? <App />}</React.StrictMode>);