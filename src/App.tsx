import { Titlebar } from "./components/Titlebar";
import { Dashboard } from "./components/Dashboard";
import { ConnectionsView } from "./components/ConnectionsView";
import { GlobalDialog } from "./components/ui/GlobalDialog";
import { useConnections } from "./hooks/useConnections";

function App() {
  const { activeConnectionId } = useConnections();

  // The local engine lifecycle is now intelligently managed by Dashboard.tsx 
  // based on the active connection and saved configurations.

  return (
    <div class="flex flex-col h-screen overflow-hidden bg-cream">
      <Titlebar />
      <main class="flex-1 min-h-0 relative flex">
        {activeConnectionId() ? <Dashboard /> : <ConnectionsView />}
      </main>
      <GlobalDialog />
    </div>
  );
}

export default App;
