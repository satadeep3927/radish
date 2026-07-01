import { HelpCircle, Book } from "lucide-solid";
import { FAQ_ITEMS } from "../../constants/help";
import { For } from "solid-js";

export function HelpView() {
  const parseAnswer = (text: string) => {
    // Split by markdown bold (**text**) or inline code (`code`)
    const regex = /(\*\*.*?\*\*|`.*?`)/g;
    const parts = text.split(regex);
    
    return parts.map((part) => {
      if (part.startsWith("**") && part.endsWith("**")) {
        return <strong class="font-semibold text-[var(--color-text-primary)]">{part.slice(2, -2)}</strong>;
      }
      if (part.startsWith("`") && part.endsWith("`")) {
        const code = part.slice(1, -1);
        const isBrandHighlight = ["valkey-cli", "redis-cli", "ioredis", "dump.radish"].includes(code);
        return (
          <code 
            class={`bg-[var(--color-surface-2)] px-1.5 py-0.5 rounded text-xs font-mono border border-[var(--color-border-strong)] ${
              isBrandHighlight ? "text-[var(--color-brand)] font-medium" : "text-[var(--color-text-primary)]"
            }`}
          >
            {code}
          </code>
        );
      }
      return part;
    });
  };

  return (
    <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface-0)] text-[var(--color-text-primary)]">
      {/* Header */}
      <div class="flex items-center justify-between px-6 py-4 slick-border-b bg-[var(--color-surface-1)]">
        <div class="flex items-center gap-2 text-[var(--color-text-secondary)] font-sans">
          <HelpCircle class="w-5 h-5 text-[var(--color-brand)]" />
          <span class="font-medium text-[var(--color-text-primary)]">Help & Documentation</span>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-6 max-w-4xl mx-auto w-full space-y-6">
        
        {/* About Box */}
        <div class="text-center mb-8">
          <div class="w-16 h-16 bg-red-50 text-[var(--color-brand)] rounded-full flex items-center justify-center mx-auto mb-4 border border-red-100">
            <Book class="w-8 h-8" />
          </div>
          <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">Radish Studio</h2>
          <p class="text-[13px] text-[var(--color-text-secondary)] mt-1 max-w-lg mx-auto leading-relaxed">
            The lightweight, lightning-fast in-memory database engine for local development. Radish aims to provide a zero-configuration cache layer completely Valkey compatible.
          </p>
          <div class="mt-4 inline-flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
            <span>Version 0.1.0-alpha</span>
          </div>
        </div>

        {/* FAQ Section */}
        <div class="mt-8">
          <h3 class="text-sm font-semibold mb-4 text-[var(--color-text-primary)] border-b border-[var(--color-border-strong)] pb-2 uppercase tracking-wider text-xs">
            Frequently Asked Questions
          </h3>
          <div class="space-y-4">
            <For each={FAQ_ITEMS}>
              {(faq) => (
                <div class="bg-[var(--color-surface-1)] rounded border border-[var(--color-border-strong)] p-4 shadow-sm hover:shadow-md transition-shadow">
                  <h4 class="text-xs font-bold text-[var(--color-text-primary)] mb-2 uppercase tracking-wide">
                    {faq.question}
                  </h4>
                  <p class="text-[13px] text-[var(--color-text-secondary)] leading-relaxed">
                    {parseAnswer(faq.answer)}
                  </p>
                </div>
              )}
            </For>
          </div>
        </div>

      </div>
    </div>
  );
}
