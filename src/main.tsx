import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "@/App";
import { Overlay } from "@/components/overlay";
import { Splash } from "@/components/splash";
import "@/index.css";

const VIEWS: Record<string, React.ReactNode> = {
	overlay: <Overlay />,
	splash: <Splash />,
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<React.StrictMode>{VIEWS[getCurrentWebviewWindow().label] ?? <App />}</React.StrictMode>);