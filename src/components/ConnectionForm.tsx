import { createForm, zodForm } from "@modular-forms/solid";
import { z } from "zod";
import { Button } from "./ui/button";
import { Input } from "./ui/input";

const connectionSchema = z.object({
  alias: z.string().min(1, "Alias is required"),
  host: z.string().min(1, "Host is required"),
  port: z.number().int().min(1).max(65535, "Invalid port"),
});

type ConnectionFormValues = z.infer<typeof connectionSchema>;

interface ConnectionFormProps {
  onConnect: (alias: string, host: string, port: number) => void;
  isLoading: boolean;
  onCancel?: () => void;
}

export function ConnectionForm(props: ConnectionFormProps) {
  const [, { Form, Field }] = createForm<ConnectionFormValues>({
    // @ts-expect-error — Zod v4 type incompatibility with @modular-forms/solid
    validate: zodForm(connectionSchema),
    initialValues: {
      alias: "",
      host: "127.0.0.1",
      port: 6379,
    }
  });

  return (
    <Form
      class="flex flex-col gap-6 w-full max-w-sm"
      onSubmit={(values) => props.onConnect(values.alias, values.host, values.port)}
    >
      <div class="space-y-4">
        <Field name="alias">
          {(field, fieldProps) => (
            <div class="flex flex-col gap-2">
              <label class="font-bold text-[var(--color-text-primary)] uppercase tracking-wide text-xs">Alias</label>
              <Input
                {...fieldProps}
                type="text"
                value={field.value}
                placeholder="e.g. Production Cache"
                required
              />
              {field.error && <span class="text-red-500 font-bold text-xs">{field.error}</span>}
            </div>
          )}
        </Field>

        <Field name="host">
          {(field, fieldProps) => (
            <div class="flex flex-col gap-2">
              <label class="font-bold text-[var(--color-text-primary)] uppercase tracking-wide text-xs">Host</label>
              <Input
                {...fieldProps}
                type="text"
                value={field.value}
                placeholder="127.0.0.1"
                required
              />
              {field.error && <span class="text-red-500 font-bold text-xs">{field.error}</span>}
            </div>
          )}
        </Field>

        <Field name="port" type="number">
          {(field, fieldProps) => (
            <div class="flex flex-col gap-2">
              <label class="font-bold text-[var(--color-text-primary)] uppercase tracking-wide text-xs">Port</label>
              <Input
                {...fieldProps}
                type="number"
                value={field.value}
                placeholder="6379"
                required
              />
              {field.error && <span class="text-red-500 font-bold text-xs">{field.error}</span>}
            </div>
          )}
        </Field>
      </div>

      <div class="flex justify-end gap-3 mt-4">
        {props.onCancel && (
          <Button type="button" variant="ghost" onClick={props.onCancel} disabled={props.isLoading}>
            Cancel
          </Button>
        )}
        <Button type="submit" disabled={props.isLoading} size="default">
          {props.isLoading ? "Connecting..." : "Add Database"}
        </Button>
      </div>
    </Form>
  );
}
