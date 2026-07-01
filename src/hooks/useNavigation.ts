import { createSignal } from "solid-js";

export const [activeView, setActiveView] = createSignal("keys");

export function useNavigation() {
  return { activeView, setActiveView };
}
