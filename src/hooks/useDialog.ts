import { createSignal } from "solid-js";

interface AlertOptions {
  title: string;
  message: string;
  confirmText?: string;
  variant?: "danger" | "default";
  onConfirm?: () => void;
}

interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: "danger" | "default";
  onConfirm: () => void;
  onCancel?: () => void;
}

interface PromptOptions {
  title: string;
  label?: string;
  defaultValue?: string;
  confirmText?: string;
  onConfirm: (value: string) => void;
  onCancel?: () => void;
}

export type DialogState =
  | { type: "alert"; options: AlertOptions }
  | { type: "confirm"; options: ConfirmOptions }
  | { type: "prompt"; options: PromptOptions }
  | null;

// Module-level singleton — any component can import and use this
const [dialogState, setDialogState] = createSignal<DialogState>(null);

export function useDialog() {
  const alert = (options: AlertOptions) => {
    setDialogState({ type: "alert", options });
  };

  const confirm = (options: ConfirmOptions) => {
    setDialogState({ type: "confirm", options });
  };

  const prompt = (options: PromptOptions) => {
    setDialogState({ type: "prompt", options });
  };

  const close = () => setDialogState(null);

  return { dialogState, alert, confirm, prompt, close };
}
