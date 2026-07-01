/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import App from "./App";

// Disable browser context menu (right-click)
document.addEventListener("contextmenu", (e) => e.preventDefault());

const INPUT_TAGS = new Set(["INPUT", "TEXTAREA", "SELECT"]);

// Disable browser shortcuts in the desktop app, but not inside form fields
document.addEventListener("keydown", (e) => {
  const target = e.target as HTMLElement | null;
  if (target && INPUT_TAGS.has(target.tagName) && (target as HTMLInputElement).readOnly !== true) {
    return;
  }
  // Ctrl+R / F5 — reload
  if (e.key === "F5" || ((e.ctrlKey || e.metaKey) && e.key === "r")) {
    e.preventDefault();
  }
  // Ctrl+S — save page
  if ((e.ctrlKey || e.metaKey) && e.key === "s") {
    e.preventDefault();
  }
});

document.getElementById("radish-loader")?.remove();
render(() => <App />, document.getElementById("root") as HTMLElement);
