import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "@/App";
import { Overlay } from "@/components/overlay";
import "@/index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<React.StrictMode>
		{getCurrentWebviewWindow().label === "overlay" ? <Overlay /> : <App />}
	</React.StrictMode>,
);