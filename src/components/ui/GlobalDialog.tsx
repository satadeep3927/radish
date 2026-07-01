import { createSignal, Show, JSX } from "solid-js";
import { useDialog, type DialogState } from "../../hooks/useDialog";
import { X, AlertTriangle } from "lucide-solid";

function AlertDialog(props: { state: NonNullable<DialogState> & { type: "alert" }; onClose: () => void }) {
  const isDanger = () => props.state.options.variant === "danger";
  return (
    <DialogShell isDanger={isDanger()} title={props.state.options.title} onClose={props.onClose}
      footer={
        <DialogFooter confirmText={props.state.options.confirmText || "OK"} onConfirm={() => { props.state.options.onConfirm?.(); props.onClose(); }} isDanger={isDanger()} hideCancel />
      }
    >
      <p class="text-sm text-[var(--color-text-secondary)]">{props.state.options.message}</p>
    </DialogShell>
  );
}

function ConfirmDialog(props: { state: NonNullable<DialogState> & { type: "confirm" }; onClose: () => void }) {
  const isDanger = () => props.state.options.variant === "danger";

  return (
    <DialogShell isDanger={isDanger()} title={props.state.options.title} onClose={props.onClose}
      footer={
        <DialogFooter
          confirmText={props.state.options.confirmText || "OK"}
          cancelText={props.state.options.cancelText}
          onConfirm={() => { props.state.options.onConfirm(); props.onClose(); }}
          onCancel={() => { props.state.options.onCancel?.(); props.onClose(); }}
          isDanger={isDanger()}
        />
      }
    >
      <p class="text-sm text-[var(--color-text-secondary)]">{props.state.options.message}</p>
    </DialogShell>
  );
}

function PromptDialog(props: { state: NonNullable<DialogState> & { type: "prompt" }; onClose: () => void }) {
  const [value, setValue] = createSignal(props.state.options.defaultValue || "");

  const handleConfirm = () => {
    props.state.options.onConfirm(value());
    props.onClose();
  };

  return (
    <DialogShell isDanger={false} title={props.state.options.title} onClose={props.onClose}
      footer={
        <DialogFooter
          confirmText={props.state.options.confirmText || "OK"}
          cancelText="Cancel"
          onConfirm={handleConfirm}
          onCancel={() => { props.state.options.onCancel?.(); props.onClose(); }}
          isDanger={false}
        />
      }
    >
      {props.state.options.label && <p class="text-xs text-[var(--color-text-secondary)] mb-3">{props.state.options.label}</p>}
      <input
        ref={(el) => setTimeout(() => el?.focus(), 30)}
        type="text"
        value={value()}
        onInput={(e) => setValue(e.currentTarget.value)}
        onKeyDown={(e) => e.key === "Enter" && handleConfirm()}
        class="w-full bg-[var(--color-surface-1)] border border-[var(--color-border-strong)] rounded px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-brand)] focus:ring-1 focus:ring-[var(--color-brand)]/20 transition-all text-[var(--color-text-primary)] font-mono"
      />
    </DialogShell>
  );
}

function DialogShell(props: {
  isDanger: boolean;
  title: string;
  onClose: () => void;
  children: JSX.Element;
  footer: JSX.Element;
}) {
  return (
    <div
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={(e) => e.target === e.currentTarget && props.onClose()}
    >
      <div class="bg-[var(--color-surface-0)] w-full max-w-sm rounded-xl border border-[var(--color-border-strong)] shadow-2xl overflow-hidden animate-in">
        <div class="flex items-center justify-between px-5 py-3.5 slick-border-b bg-[var(--color-surface-1)]">
          <div class="flex items-center gap-2.5">
            {props.isDanger && <AlertTriangle class="w-4 h-4 text-[var(--color-brand)]" />}
            <h2 class="text-sm font-semibold text-[var(--color-text-primary)]">{props.title}</h2>
          </div>
          <button onClick={props.onClose} class="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] transition-colors">
            <X class="w-4 h-4" />
          </button>
        </div>
        <div class="px-5 py-4">{props.children}</div>
        <div class="flex items-center justify-end gap-2 px-5 py-3 slick-border-t bg-[var(--color-surface-1)]">
          {props.footer}
        </div>
      </div>
    </div>
  );
}

function DialogFooter(props: {
  confirmText: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel?: () => void;
  isDanger: boolean;
  hideCancel?: boolean;
}) {
  return (
    <div class="flex items-center justify-end gap-2">
      {!props.hideCancel && (
        <button
          onClick={props.onCancel}
          class="px-3 py-1.5 text-xs rounded border border-[var(--color-border-strong)] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-2)] transition-colors"
        >
          {props.cancelText || "Cancel"}
        </button>
      )}
      <button
        onClick={props.onConfirm}
        class={`px-3 py-1.5 text-xs rounded font-medium transition-colors text-white ${
          props.isDanger ? "bg-[var(--color-brand)] hover:bg-red-700" : "bg-[var(--color-text-primary)] hover:opacity-80"
        }`}
      >
        {props.confirmText}
      </button>
    </div>
  );
}

export function GlobalDialog() {
  const { dialogState, close } = useDialog();

  return (
    <Show when={dialogState()}>
      {(s) => {
        const state = s();
        if (state.type === "alert") return <AlertDialog state={state} onClose={close} />;
        if (state.type === "confirm") return <ConfirmDialog state={state} onClose={close} />;
        if (state.type === "prompt") return <PromptDialog state={state} onClose={close} />;
        return null;
      }}
    </Show>
  );
}
