<script lang="ts">
  import { onMount } from "svelte";
  import { workspace } from "./workspaceStore.svelte";

  let bind = $state("127.0.0.1");
  let port = $state(4000);
  let busy = $state(false);

  async function start() {
    busy = true;
    await workspace.startMock(bind, port);
    busy = false;
  }

  async function stop() {
    busy = true;
    await workspace.stopMock();
    busy = false;
  }

  onMount(() => {
    const id = window.setInterval(() => {
      if (workspace.mockServer?.running) workspace.refreshMockServer();
    }, 1200);
    return () => window.clearInterval(id);
  });
</script>

{#if workspace.info}
  <section class="border-b border-neutral-900 bg-neutral-950/70 px-4 py-3">
    <div class="flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2 text-[11px] uppercase tracking-widest text-neutral-600">
        <span class="h-2 w-2 rounded-full {workspace.mockServer?.running ? 'bg-emerald-500' : 'bg-neutral-700'}"></span>
        Mock server
      </div>

      <input
        class="w-32 rounded border border-neutral-800 bg-neutral-950 px-2 py-1 text-xs text-neutral-200 focus:border-indigo-600 focus:outline-none"
        bind:value={bind}
        disabled={workspace.mockServer?.running || busy}
        aria-label="Mock server bind address"
      />

      <input
        class="w-20 rounded border border-neutral-800 bg-neutral-950 px-2 py-1 text-xs text-neutral-200 focus:border-indigo-600 focus:outline-none"
        type="number"
        min="1"
        max="65535"
        bind:value={port}
        disabled={workspace.mockServer?.running || busy}
        aria-label="Mock server port"
      />

      {#if workspace.mockServer?.running}
        <button
          class="rounded border border-rose-900/70 px-3 py-1 text-xs text-rose-200 hover:border-rose-700 disabled:opacity-50"
          onclick={stop}
          disabled={busy}
        >
          Stop
        </button>
        <span class="text-xs text-neutral-500">
          {workspace.mockServer.url} · {workspace.mockServer.rules} rules
        </span>
      {:else}
        <button
          class="rounded border border-emerald-900/70 px-3 py-1 text-xs text-emerald-200 hover:border-emerald-700 disabled:opacity-50"
          onclick={start}
          disabled={busy}
        >
          Start
        </button>
      {/if}
    </div>

    {#if workspace.mockServer?.running}
      <div class="mt-3 max-h-32 overflow-auto border-t border-neutral-900 pt-2 font-mono text-[11px] text-neutral-400">
        {#if workspace.mockServer.logs.length === 0}
          <div class="text-neutral-600">No requests yet.</div>
        {:else}
          {#each workspace.mockServer.logs.slice().reverse() as log}
            <div class="grid grid-cols-[56px_44px_minmax(0,1fr)_minmax(8rem,auto)] gap-2 py-0.5">
              <span class="text-neutral-500">{log.method}</span>
              <span class={log.status >= 500 ? "text-rose-300" : log.status >= 400 ? "text-amber-300" : "text-emerald-300"}>{log.status}</span>
              <span class="truncate text-neutral-300">{log.path}</span>
              <span class="truncate text-neutral-600">{log.matched_rule ?? "(no match)"}</span>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </section>
{/if}
