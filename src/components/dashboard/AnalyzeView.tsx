import { createSignal, Show, For } from "solid-js";
import { useKeys } from "../../hooks/useKeys";
import { ChevronDown, Play } from "lucide-solid";
import { useDatabaseStats } from "../../hooks/useDatabaseStats";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { formatBytes } from "../../lib/format";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "../ui/table";

export function AnalyzeView() {
  const { keys, keyTypes, fetchKeys } = useKeys();
  const [expanded, setExpanded] = createSignal<Set<string>>(new Set());
  const [reportTime, setReportTime] = createSignal(new Date());

  const {
    isFetchingSizes,
    nsSort,
    setNsSort,
    topSort,
    setTopSort,
    stats,
    sortedNamespaces,
    topKeys,
    getConicGradient
  } = useDatabaseStats(keys, keyTypes);

  const handleNewReport = () => {
    fetchKeys();
    setReportTime(new Date());
  };

  const toggleExpanded = (ns: string) => {
    const next = new Set(expanded());
    if (next.has(ns)) next.delete(ns);
    else next.add(ns);
    setExpanded(next);
  };

  const formatMem = formatBytes;

  const nowStr = () => reportTime().toLocaleString('en-GB', { 
    day: '2-digit', month: 'short', year: 'numeric', 
    hour: '2-digit', minute: '2-digit', second: '2-digit' 
  });

  return (
    <div class="flex flex-col flex-1 min-h-0 bg-[var(--color-surface-0)]">
      {/* Top Header */}
      <div class="flex items-center justify-between px-6 pt-4 pb-4 slick-border-b bg-[var(--color-surface-2)]">
        <div class="flex items-center gap-6">
          <h2 class="text-lg font-bold text-[var(--color-text-primary)]">Database Analysis</h2>
        </div>
        <div class="flex items-center gap-4">
          <div class="text-xs text-[var(--color-text-secondary)]">
            Report generated on: <span class="font-medium text-[var(--color-text-primary)] ml-1">{nowStr()}</span>
          </div>
          <div class="text-xs text-[#00A859]">
            Scanned 100% ({stats().total}/{stats().total} keys)
          </div>
          <Button onClick={handleNewReport} class="h-8 text-xs font-medium bg-[var(--color-brand)] hover:bg-[var(--color-brand-hover)] text-white shadow-sm flex items-center gap-1.5 px-3">
            <Play class="w-3 h-3 fill-current" />
            New Report
          </Button>
        </div>
      </div>

      {/* Main Content Area */}
      <div class="flex-1 overflow-y-auto p-6 flex flex-col gap-10">
        
        {/* SUMMARY PER DATA TYPE */}
        <div>
          <div class="flex items-center gap-4 mb-8">
            <h3 class="text-[13px] font-bold text-[var(--color-text-primary)] uppercase tracking-wider">Summary Per Data Type</h3>
          </div>

          <div class="flex justify-center my-6">
            <div class="relative flex items-center justify-center w-[240px] h-[240px]">
              {/* Labels positioned absolutely around the donut */}
              <Show when={stats().stringPct > 0}>
                <div class="absolute -right-16 top-1/2 -translate-y-1/2 text-xs font-semibold text-[var(--color-text-primary)]">String: {stats().stringPct.toFixed(0)}%</div>
              </Show>
              <Show when={stats().hashPct > 0}>
                <div class="absolute -left-16 bottom-10 text-xs font-semibold text-[var(--color-text-primary)]">Hash: {stats().hashPct.toFixed(0)}%</div>
              </Show>
              <Show when={stats().listPct > 0}>
                <div class="absolute -left-16 top-1/4 text-xs font-semibold text-[var(--color-text-primary)]">List: {stats().listPct.toFixed(0)}%</div>
              </Show>
              <Show when={stats().setPct > 0}>
                <div class="absolute left-1/2 -top-6 -translate-x-1/2 text-xs font-semibold text-[var(--color-text-primary)]">Set: {stats().setPct.toFixed(0)}%</div>
              </Show>

              {/* The Donut */}
              <div 
                class="w-[200px] h-[200px] rounded-full flex items-center justify-center"
                style={{ background: `conic-gradient(${getConicGradient()})` }}
              >
                {/* Donut Inner Hole */}
                <div class="w-[188px] h-[188px] bg-white rounded-full flex flex-col items-center justify-center relative">
                  <div class="flex items-center gap-1.5 text-[var(--color-text-primary)] mb-1">
                    <svg class="w-4 h-4 text-[var(--color-text-muted)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>
                    <span class="text-sm font-semibold">Keys</span>
                  </div>
                  <div class="text-base font-medium text-[var(--color-text-primary)]">~{stats().total}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* TOP NAMESPACES */}
        <div>
          <div class="flex items-center gap-4 mb-4">
            <h3 class="text-[13px] font-bold text-[var(--color-text-primary)] uppercase tracking-wider">Top Namespaces</h3>
            <div class="flex items-center gap-3 ml-2 text-xs text-[var(--color-text-muted)]">
              <button class={`transition-colors ${nsSort() === "memory" ? "font-medium text-[var(--color-brand)]" : "hover:text-[var(--color-text-primary)]"}`} onClick={() => setNsSort("memory")}>by Memory</button>
              <button class={`transition-colors ${nsSort() === "keys" ? "font-medium text-[var(--color-brand)]" : "hover:text-[var(--color-text-primary)]"}`} onClick={() => setNsSort("keys")}>by Number of Keys</button>
            </div>
          </div>

          <div class="border border-[var(--color-border-strong)] rounded-lg overflow-hidden bg-[var(--color-surface-1)] shadow-sm">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead class="px-4 py-3 w-1/3">Namespace</TableHead>
                  <TableHead class="px-4 py-3 w-1/4">Keys</TableHead>
                  <TableHead class="px-4 py-3 w-1/4">Memory</TableHead>
                  <TableHead class="px-4 py-3">Data Types</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <For each={sortedNamespaces()}>
                  {(ns) => (
                    <>
                      <TableRow 
                        class="cursor-pointer"
                        onClick={() => toggleExpanded(ns.name)}
                      >
                        <TableCell class="px-4 py-3 text-sm font-medium text-[var(--color-text-primary)] font-mono flex items-center gap-2">
                          <ChevronDown class={`w-4 h-4 text-[var(--color-text-muted)] transition-transform ${expanded().has(ns.name) ? '' : '-rotate-90'}`} />
                          {ns.name}
                        </TableCell>
                        <TableCell class="px-4 py-3 text-sm font-mono text-[var(--color-text-secondary)]">~ {ns.count}</TableCell>
                        <TableCell class="px-4 py-3 text-sm font-mono text-[var(--color-text-secondary)]">~ {formatMem(ns.memory)}</TableCell>
                        <TableCell class="px-4 py-3 text-sm flex items-center gap-1.5 flex-wrap font-sans">
                          <For each={ns.types}>
                            {(t) => (
                              <Badge type={t} class="text-[10px]" />
                            )}
                          </For>
                        </TableCell>
                      </TableRow>
                      <Show when={expanded().has(ns.name)}>
                        <TableRow class="bg-[var(--color-surface-2)] slick-border-b inset-shadow hover:bg-transparent">
                          <TableCell colspan="4" class="px-10 py-3 text-xs font-mono text-[var(--color-text-secondary)]">
                            <div class="max-h-40 overflow-y-auto pr-4">
                              <For each={ns.nestedKeys}>
                                {(k) => <div class="py-1 slick-border-b border-white/50 last:border-0 hover:text-[var(--color-brand)] transition-colors">{k}</div>}
                              </For>
                            </div>
                          </TableCell>
                        </TableRow>
                      </Show>
                    </>
                  )}
                </For>
              </TableBody>
            </Table>
          </div>
        </div>

        {/* TOP KEYS */}
        <div>
          <div class="flex items-center gap-4 mb-4">
            <h3 class="text-[13px] font-bold text-[var(--color-text-primary)] uppercase tracking-wider">Top Keys</h3>
            <div class="flex items-center gap-3 ml-2 text-xs text-[var(--color-text-muted)]">
              <button class={`transition-colors ${topSort() === "size" ? "font-medium text-[var(--color-brand)]" : "hover:text-[var(--color-text-primary)]"}`} onClick={() => setTopSort("size")}>by Size</button>
              <button class={`transition-colors ${topSort() === "length" ? "font-medium text-[var(--color-brand)]" : "hover:text-[var(--color-text-primary)]"}`} onClick={() => setTopSort("length")}>by Length</button>
            </div>
          </div>
          
          <div class="border border-[var(--color-border-strong)] rounded-lg overflow-hidden bg-[var(--color-surface-1)] shadow-sm">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead class="px-4 py-3 w-1/3">Key</TableHead>
                  <TableHead class="px-4 py-3 w-1/6">Type</TableHead>
                  <TableHead class="px-4 py-3 w-1/6">TTL</TableHead>
                  <TableHead class="px-4 py-3 w-1/6">Size</TableHead>
                  <TableHead class="px-4 py-3 w-1/6">Length</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <For each={topKeys()}>
                  {(k) => (
                    <TableRow>
                      <TableCell class="px-4 py-3 text-sm font-medium text-[var(--color-text-primary)] font-mono truncate max-w-[200px]" title={k.name}>{k.name}</TableCell>
                      <TableCell class="px-4 py-3 text-sm font-mono text-[var(--color-text-secondary)] font-sans">
                        <Badge type={k.type} />
                      </TableCell>
                      <TableCell class="px-4 py-3 text-sm font-mono text-[var(--color-text-secondary)]">{k.ttl === -1 ? 'None' : k.ttl}</TableCell>
                      <TableCell class="px-4 py-3 text-sm font-mono text-[var(--color-text-secondary)]">~ {formatMem(k.size)}</TableCell>
                      <TableCell class="px-4 py-3 text-sm font-mono text-[var(--color-text-secondary)]">{k.length}</TableCell>
                    </TableRow>
                  )}
                </For>
              </TableBody>
            </Table>
          </div>
        </div>

      </div>
    </div>
  );
}
