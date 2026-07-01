import { Show, For } from "solid-js";
import { usePubSub } from "../../hooks/usePubSub";
import { Lightbulb, Trash2, ArrowDown } from "lucide-solid";
import { Button } from "../ui/button";
import { createForm, zodForm } from "@modular-forms/solid";
import { z } from "zod";

const subscribeSchema = z.object({
  channel: z.string().min(1, "Channel is required"),
});
type SubscribeForm = z.infer<typeof subscribeSchema>;

const publishSchema = z.object({
  channel: z.string().min(1, "Channel is required"),
  message: z.string().min(1, "Message is required"),
});
type PublishForm = z.infer<typeof publishSchema>;

export function PubSubView(props: { pubsubState: ReturnType<typeof usePubSub> }) {
  const {
    activeChannels,
    messages,
    subscribe,
    unsubscribe,
    publish,
    clearMessages,
  } = props.pubsubState;

  const isSubscribed = () => activeChannels().length > 0;

  const [, { Form: SubForm, Field: SubField, reset: resetSub }] = createForm<SubscribeForm>({
    validate: zodForm(subscribeSchema),
    initialValues: { channel: "" }
  });

  const [pubForm, { Form: PubForm, Field: PubField, reset: resetPub }] = createForm<PublishForm>({
    validate: zodForm(publishSchema),
    initialValues: { channel: "", message: "" }
  });

  const handleSubscribe = (values: SubscribeForm) => {
    subscribe(values.channel);
    resetSub();
  };

  const handlePublish = (values: PublishForm) => {
    publish(values.channel, values.message);
    resetPub({ initialValues: { channel: values.channel, message: "" } });
  };

  const handleUnsubscribe = () => {
    const channels = activeChannels();
    if (channels.length > 0) {
      channels.forEach(ch => unsubscribe(ch));
    }
  };

  return (
    <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface-0)] text-[var(--color-text-primary)]">
      <Show
        when={isSubscribed()}
        fallback={
          <div class="flex-1 flex flex-col items-center justify-center p-8 overflow-y-auto">
            <div class="max-w-md w-full text-center flex flex-col items-center">
              <div class="w-24 h-24 mb-6 relative">
                <Lightbulb class="w-24 h-24 text-[var(--color-brand)] drop-shadow-md" stroke-width={1} />
              </div>
              <h2 class="text-2xl font-semibold text-[var(--color-text-primary)] mb-2">You are not subscribed</h2>
              <p class="text-[var(--color-text-secondary)] mb-8 text-sm">
                Subscribe to a Channel to see all the messages published to your database
              </p>

              <SubForm 
                onSubmit={handleSubscribe}
                class="flex items-center gap-2 w-full mb-6"
              >
                <SubField name="channel">
                  {(field, fieldProps) => (
                    <div class="flex-1 flex flex-col text-left gap-1">
                      <input
                        {...fieldProps}
                        type="text"
                        value={field.value}
                        placeholder="Enter Channel Name (e.g. *)"
                        class="w-full h-9 px-3 rounded border border-[var(--color-border-strong)] bg-[var(--color-surface-2)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-brand)] transition-colors"
                      />
                      {field.error && <span class="text-red-500 font-semibold text-[10px]">{field.error}</span>}
                    </div>
                  )}
                </SubField>
                <Button type="submit" class="h-9 px-6 bg-[var(--color-brand)] hover:bg-[var(--color-brand-hover)] text-white font-medium shadow-sm shrink-0">
                  Subscribe
                </Button>
                <Button type="button" variant="ghost" class="px-2 text-[var(--color-text-muted)] hover:text-[var(--color-brand)] shrink-0" onClick={clearMessages} title="Clear Messages">
                  <Trash2 class="w-4 h-4" />
                </Button>
              </SubForm>
            </div>
          </div>
        }
      >
        <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface-1)]">
          {/* Top Toolbar */}
          <div class="flex items-center justify-between px-6 py-4 slick-border-b">
            <div class="flex items-center gap-6 text-sm text-[var(--color-text-secondary)]">
              <div class="flex items-center gap-1.5">
                <span>Patterns:</span>
                <span class="font-medium text-[var(--color-text-primary)]">{activeChannels().join(", ")}</span>
              </div>
              <div class="flex items-center gap-1.5">
                <span>Messages:</span>
                <span class="font-medium text-[var(--color-text-primary)]">{messages().length}</span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium border border-green-500/20 text-green-400 bg-green-500/10">
                Status: Subscribed
              </div>
              
              <SubForm 
                onSubmit={handleSubscribe}
                class="flex items-center gap-2"
              >
                <SubField name="channel">
                  {(field, fieldProps) => (
                    <div class="flex flex-col gap-1">
                      <input
                        {...fieldProps}
                        type="text"
                        value={field.value}
                        placeholder="Channel Name"
                        class="h-8 px-3 w-48 rounded border border-[var(--color-border-strong)] bg-[var(--color-surface-2)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-brand)] text-sm transition-colors"
                      />
                    </div>
                  )}
                </SubField>
                <Button type="submit" class="h-8 px-3 bg-[var(--color-brand)] hover:bg-[var(--color-brand-hover)] text-white text-sm font-medium">
                  Subscribe
                </Button>
                <Button type="button" onClick={handleUnsubscribe} class="h-8 px-3 bg-transparent border border-[var(--color-border-strong)] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-2)] font-medium shadow-sm flex items-center gap-2 text-sm">
                  Unsubscribe
                </Button>
                <Button type="button" variant="ghost" class="px-2 text-[var(--color-text-muted)] hover:text-[var(--color-brand)]" onClick={clearMessages} title="Clear Messages">
                  <Trash2 class="w-4 h-4" />
                </Button>
              </SubForm>
            </div>
          </div>

          {/* Table Header */}
          <div class="grid grid-cols-12 gap-4 px-6 py-2.5 slick-border-b bg-[var(--color-surface-2)] font-medium text-xs text-[var(--color-text-secondary)] select-none uppercase tracking-wider">
            <div class="col-span-3 flex items-center gap-1 cursor-pointer hover:text-[var(--color-text-primary)] transition-colors">
              Timestamp <ArrowDown class="w-3 h-3" />
            </div>
            <div class="col-span-3">Channel</div>
            <div class="col-span-6">Message</div>
          </div>

          {/* Table Body */}
          <div class="flex-1 overflow-y-auto min-h-0 bg-[var(--color-surface-0)]">
            <Show 
              when={messages().length > 0} 
              fallback={<div class="p-8 text-center text-[var(--color-text-muted)] text-sm">No messages published yet</div>}
            >
              <For each={messages().slice().reverse()}>
                {(msg) => (
                  <div class="grid grid-cols-12 gap-4 px-6 py-2.5 slick-border-b hover:bg-[var(--color-surface-2)] text-sm transition-colors group">
                    <div class="col-span-3 text-[var(--color-text-muted)] font-mono text-xs flex items-center group-hover:text-[var(--color-text-secondary)] transition-colors">{msg.time}</div>
                    <div class="col-span-3 text-[var(--color-text-secondary)] font-medium truncate flex items-center group-hover:text-[var(--color-text-primary)] transition-colors" title={msg.channel}>{msg.channel}</div>
                    <div class="col-span-6 text-[var(--color-text-primary)] font-mono text-xs break-all flex items-center">{msg.data}</div>
                  </div>
                )}
              </For>
            </Show>
          </div>
        </div>
      </Show>

      {/* Sticky Bottom Publish Bar */}
      <div class="slick-border-t bg-[var(--color-surface-1)] p-4 shadow-lg z-10">
        <PubForm onSubmit={handlePublish} class="flex items-end gap-3 max-w-[1400px] mx-auto w-full">
          <div class="flex-1 max-w-[240px]">
            <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5 uppercase tracking-wider">Channel name</label>
            <PubField name="channel">
              {(field, fieldProps) => (
                <div class="flex flex-col gap-1">
                  <input
                    {...fieldProps}
                    type="text"
                    value={field.value}
                    placeholder="Enter Channel Name"
                    class="w-full h-9 px-3 rounded border border-[var(--color-border-strong)] bg-[var(--color-surface-2)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-brand)] text-sm transition-colors"
                  />
                </div>
              )}
            </PubField>
          </div>
          <div class="flex-1">
            <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5 uppercase tracking-wider">Message</label>
            <PubField name="message">
              {(field, fieldProps) => (
                <div class="flex flex-col gap-1">
                  <input
                    {...fieldProps}
                    type="text"
                    value={field.value}
                    placeholder="Enter Message"
                    class="w-full h-9 px-3 rounded border border-[var(--color-border-strong)] bg-[var(--color-surface-2)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-brand)] text-sm transition-colors"
                  />
                </div>
              )}
            </PubField>
          </div>
          <Button 
            type="submit" 
            disabled={pubForm.submitting}
            class="h-9 px-8 bg-[var(--color-text-primary)] hover:opacity-90 text-[var(--color-surface-0)] font-medium shadow-sm disabled:opacity-30 disabled:cursor-not-allowed transition-all shrink-0"
          >
            Publish
          </Button>
        </PubForm>
      </div>
    </div>
  );
}
