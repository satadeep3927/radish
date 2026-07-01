import { onMount, Show, Switch, Match, createEffect, on, Accessor } from "solid-js";
import { Power } from "lucide-solid";
import { Button } from "./ui/button";

import { Sidebar } from "./dashboard/Sidebar";
import { StatsRow } from "./dashboard/StatsRow";
import { KeysView } from "./dashboard/KeysView";
import { AnalyzeView } from "./dashboard/AnalyzeView";
import { PubSubView } from "./dashboard/PubSubView";
import { CliView } from "./dashboard/CliView";
import { ServiceView } from "./dashboard/ServiceView";
import { ConfigView } from "./dashboard/ConfigView";
import { HelpView } from "./dashboard/HelpView";

import { useRadish } from "../hooks/useRadish";
import { useKeys } from "../hooks/useKeys";
import { usePubSub } from "../hooks/usePubSub";
import { useCli } from "../hooks/useCli";
import { useNavigation } from "../hooks/useNavigation";
import { useConnections } from "../hooks/useConnections";

type Accessor<T> = () => T;

function handleAutoConnect(checkConnection: () => Promise<void>, startServer: () => Promise<void>, isConnected: Accessor<boolean>, activeConnectionId: Accessor<string>) {
  return async () => {
    await checkConnection();
    if (!isConnected() && activeConnectionId() === "local-radish-engine") {
      await startServer();
    }
  };
}

function handleStartServer(startServer: () => Promise<void>) {
  return () => {
    startServer();
  };
}

function DisconnectedFallback(props: {
  isLocalStopped: Accessor<boolean>;
  connectionError: Accessor<string | null>;
  onRetry: () => void;
  onStart: () => void;
}) {
  return (
    <div class="flex flex-col items-center justify-center h-full w-full bg-[var(--surface-0)] text-[var(--color-text-primary)]">
      <div class="p-6 bg-[var(--color-surface-1)] rounded-xl border border-[var(--color-border-strong)] text-center max-w-sm">
        <div class="w-12 h-12 bg-[var(--color-brand-subtle)] text-[var(--color-brand)] rounded-full flex items-center justify-center mx-auto mb-4">
          <Power class="w-6 h-6" />
        </div>
        <Show
          when={props.isLocalStopped()}
          fallback={
            <>
              <h2 class="text-lg font-semibold mb-2">Connection Failed</h2>
              <p class="text-sm text-[var(--color-text-secondary)] mb-6">
                Could not connect to the remote database. Make sure the server is running and accessible.
                {props.connectionError() && <span class="block mt-2 text-xs text-[var(--color-brand)]">{props.connectionError()}</span>}
              </p>
              <Button onClick={props.onRetry} class="w-full bg-[var(--color-brand)] hover:bg-[var(--color-brand-hover)] text-white">
                Retry Connection
              </Button>
            </>
          }
        >
          <h2 class="text-lg font-semibold mb-2">Radish Server Stopped</h2>
          <p class="text-sm text-[var(--color-text-secondary)] mb-6">
            The local Radish engine is currently stopped.
            {props.connectionError() && <span class="block mt-2 text-xs text-[var(--color-brand)]">{props.connectionError()}</span>}
          </p>
          <Button onClick={props.onStart} class="w-full bg-[var(--color-brand)] hover:bg-[var(--color-brand-hover)] text-white">
            Start Server
          </Button>
        </Show>
      </div>
    </div>
  );
}

export function Dashboard() {
  const { activeView, setActiveView } = useNavigation();
  const { isConnected, startServer, stopServer, checkConnection, connectionError, isEngineOwnedByStudio } = useRadish();
  const { keys, keyTypes, fetchKeys } = useKeys();
  const { activeConnectionId } = useConnections();
  const pubsubState = usePubSub();
  const cliState = useCli();

  createEffect(on(activeConnectionId, () => {
    if ((activeView() === "service" || activeView() === "config") && !isEngineOwnedByStudio()) {
      setActiveView("keys");
    }
  }, { defer: true }));

  onMount(handleAutoConnect(checkConnection, startServer, isConnected, activeConnectionId));

  const isLocalStopped = () => activeConnectionId() === "local-radish-engine" && !isEngineOwnedByStudio();

  return (
    <Show
      when={isConnected()}
      fallback={
        <DisconnectedFallback
          isLocalStopped={isLocalStopped}
          connectionError={connectionError}
          onRetry={checkConnection}
          onStart={handleStartServer(startServer)}
        />
      }
    >
      <div class="flex h-full w-full min-h-0 bg-[var(--surface-0)] text-[var(--color-text-primary)]">
        <Sidebar activeView={activeView()} setActiveView={setActiveView} keyCount={keys().length} onStopServer={() => stopServer()} isEngineOwnedByStudio={isEngineOwnedByStudio} />

      <div class="flex-1 flex flex-col min-w-0 min-h-0">
        <StatsRow />

        <Switch fallback={<div class="p-6 text-xs text-[var(--color-text-muted)]">View not implemented.</div>}>
          <Match when={activeView() === "keys"}>
            <KeysView keys={keys} keyTypes={keyTypes} fetchKeys={fetchKeys} connectionId={activeConnectionId} />
          </Match>
          <Match when={activeView() === "analyze"}>
            <AnalyzeView />
          </Match>
          <Match when={activeView() === "pubsub"}>
            <PubSubView pubsubState={pubsubState} />
          </Match>
          <Match when={activeView() === "cli"}>
            <CliView cliState={cliState} />
          </Match>
          <Match when={activeView() === "service"}>
            <ServiceView />
          </Match>
          <Match when={activeView() === "config"}>
            <ConfigView />
          </Match>
          <Match when={activeView() === "help"}>
            <HelpView />
          </Match>
        </Switch>
      </div>
      </div>
    </Show>
  );
}
