<script lang="ts">
  import type { HistoryReplay, RequestDefinition } from "./api";
  import { workspace } from "./workspaceStore.svelte";

  type Props = {
    onLoad: (def: RequestDefinition, id: string) => void;
    onReplay: (replay: HistoryReplay) => void;
  };

  let { onLoad, onReplay }: Props = $props();

  const grouped = $derived.by(() => {
    const map = new Map<string, typeof workspace.entries>();
    for (const e of workspace.entries) {
      const list = map.get(e.collection) ?? [];
      list.push(e);
      map.set(e.collection, list);
    }
    return [...map.entries()].sort(([a], [b]) => a.localeCompare(b));
  });

  async function handleClick(id: string) {
    const def = await workspace.load(id);
    if (def) onLoad(def, id);
  }

  async function handleHistoryClick(id: number) {
    const replay = await workspace.replayHistory(id);
    if (replay) onReplay(replay);
  }

  async function pickWorkspace() {
    await workspace.pickAndOpen();
  }

  async function confirmClear() {
    if (confirm("Clear all request history for this workspace?")) {
      await workspace.wipeHistory();
    }
  }

  function timeAgo(tsSec: number): string {
    const diff = Math.max(0, Math.floor(Date.now() / 1000) - tsSec);
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  }

  function statusColor(s: number): string {
    if (s >= 200 && s < 300) return "text-emerald-400";
    if (s >= 300 && s < 400) return "text-sky-400";
    if (s >= 400 && s < 500) return "text-amber-400";
    return "text-red-400";
  }
</script>

<aside class="flex h-full w-60 flex-col border-r border-neutral-800 bg-neutral-950">
  <div class="flex items-center justify-between border-b border-neutral-800 px-3 py-2">
    <div class="min-w-0 flex-1">
      {#if workspace.info}
        <div class="truncate text-[11px] font-semibold text-neutral-200">
          {workspace.info.manifest.name}
        </div>
        <div class="truncate text-[10px] text-neutral-600" title={workspace.info.root}>
          {workspace.info.root}
        </div>
      {:else}
        <div class="text-[11px] text-neutral-500">No workspace open</div>
      {/if}
    </div>
    <button
      onclick={pickWorkspace}
      title="Open workspace…"
      class="ml-2 rounded border border-neutral-800 px-2 py-1 text-[10px] uppercase tracking-widest text-neutral-400 hover:border-indigo-600 hover:text-indigo-400"
    >
      Open
    </button>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#if workspace.error}
      <div class="m-2 rounded border border-red-900 bg-red-950/40 p-2 text-[11px] text-red-300">
        {workspace.error}
      </div>
    {/if}

    {#if !workspace.info}
      <div class="p-3 text-[11px] text-neutral-600">
        Open a folder to start saving requests.
      </div>
    {:else if workspace.entries.length === 0}
      <div class="p-3 text-[11px] text-neutral-600">
        No requests yet. Save one to get started.
      </div>
    {:else}
      {#each grouped as [collection, items] (collection)}
        <div class="px-2 pb-1 pt-3">
          <div class="px-1 text-[9px] uppercase tracking-widest text-neutral-500">
            {collection}
          </div>
          <ul>
            {#each items as entry (entry.id)}
              <li>
                <button
                  onclick={() => handleClick(entry.id)}
                  class="block w-full truncate rounded px-2 py-1 text-left text-[11px]
                         {workspace.activeId === entry.id
                           ? 'bg-indigo-600/20 text-indigo-200'
                           : 'text-neutral-300 hover:bg-neutral-800'}"
                  title={entry.id}
                >
                  {entry.name}
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    {/if}

    {#if workspace.info}
      <div class="mt-3 border-t border-neutral-800 px-2 pb-2 pt-3">
        <div class="flex items-center justify-between px-1">
          <div class="text-[9px] uppercase tracking-widest text-neutral-500">History</div>
          {#if workspace.history.length > 0}
            <button
              onclick={confirmClear}
              title="Clear history"
              class="text-[9px] uppercase tracking-widest text-neutral-600 hover:text-red-400"
            >
              Clear
            </button>
          {/if}
        </div>
        {#if workspace.history.length === 0}
          <div class="px-1 py-2 text-[10px] text-neutral-600">
            No requests sent yet.
          </div>
        {:else}
          <ul class="mt-1">
            {#each workspace.history as h (h.id)}
              <li>
                <button
                  onclick={() => handleHistoryClick(h.id)}
                  class="block w-full rounded px-2 py-1 text-left hover:bg-neutral-800"
                  title={h.url}
                >
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-[9px] font-semibold text-neutral-400">{h.method}</span>
                    <span class="font-mono text-[9px] {statusColor(h.status)}">{h.status}</span>
                    <span class="ml-auto text-[9px] text-neutral-600">{timeAgo(h.ts)}</span>
                  </div>
                  <div class="truncate text-[10px] text-neutral-300">{h.url}</div>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  </div>
</aside>
