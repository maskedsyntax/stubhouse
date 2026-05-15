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

  let importFlash = $state<string | null>(null);

  async function importPostmanFile() {
    const summary = await workspace.pickAndImportPostman();
    if (summary) {
      importFlash = `Imported ${summary.imported} request${summary.imported === 1 ? "" : "s"}`;
      setTimeout(() => (importFlash = null), 2500);
    }
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

<aside class="flex h-full w-64 flex-col border-r border-neutral-800 bg-neutral-950">
  <div class="flex items-center justify-between border-b border-neutral-800 px-3 py-2">
    <div class="min-w-0 flex-1">
      {#if workspace.info}
        <div class="truncate text-sm font-semibold text-neutral-100">
          {workspace.info.manifest.name}
        </div>
        <div class="truncate text-xs text-neutral-500" title={workspace.info.root}>
          {workspace.info.root}
        </div>
      {:else}
        <div class="text-sm text-neutral-400">No workspace open</div>
      {/if}
    </div>
    <div class="ml-2 flex gap-1">
      {#if workspace.info}
        <button
          onclick={importPostmanFile}
          title="Import Postman v2.1 collection (.json)"
          class="rounded border border-neutral-800 px-2.5 py-1.5 text-xs font-medium uppercase tracking-wide text-neutral-300 hover:border-indigo-600 hover:text-indigo-300"
        >
          Import
        </button>
      {/if}
      <button
        onclick={pickWorkspace}
        title="Open workspace…"
        class="rounded border border-neutral-800 px-2.5 py-1.5 text-xs font-medium uppercase tracking-wide text-neutral-300 hover:border-indigo-600 hover:text-indigo-300"
      >
        Open
      </button>
    </div>
  </div>

  {#if importFlash}
    <div class="border-b border-neutral-800 bg-emerald-950/40 px-3 py-1.5 text-xs text-emerald-200">
      {importFlash}
    </div>
  {/if}

  <div class="flex-1 overflow-y-auto">
    {#if workspace.error}
      <div class="m-2 rounded border border-red-900 bg-red-950/40 p-2 text-sm text-red-200">
        {workspace.error}
      </div>
    {/if}

    {#if !workspace.info}
      <div class="p-3 text-sm text-neutral-500">
        Open a folder to start saving requests.
      </div>
    {:else if workspace.entries.length === 0}
      <div class="p-3 text-sm text-neutral-500">
        No requests yet. Save one to get started.
      </div>
    {:else}
      {#each grouped as [collection, items] (collection)}
        <div class="px-2 pb-1 pt-3">
          <div class="px-1 text-xs font-medium uppercase tracking-wide text-neutral-400">
            {collection}
          </div>
          <ul>
            {#each items as entry (entry.id)}
              <li>
                <button
                  onclick={() => handleClick(entry.id)}
                  class="block w-full truncate rounded px-2 py-1.5 text-left text-sm
                         {workspace.activeId === entry.id
                           ? 'bg-indigo-600/20 text-indigo-200'
                           : 'text-neutral-200 hover:bg-neutral-800'}"
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
          <div class="text-xs font-medium uppercase tracking-wide text-neutral-400">History</div>
          {#if workspace.history.length > 0}
            <button
              onclick={confirmClear}
              title="Clear history"
              class="text-xs font-medium uppercase tracking-wide text-neutral-500 hover:text-red-300"
            >
              Clear
            </button>
          {/if}
        </div>
        {#if workspace.history.length === 0}
          <div class="px-1 py-2 text-xs text-neutral-500">
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
                    <span class="font-mono text-[11px] font-semibold text-neutral-300">{h.method}</span>
                    <span class="font-mono text-[11px] {statusColor(h.status)}">{h.status}</span>
                    <span class="ml-auto text-[11px] text-neutral-500">{timeAgo(h.ts)}</span>
                  </div>
                  <div class="truncate text-xs text-neutral-300">{h.url}</div>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  </div>
</aside>
