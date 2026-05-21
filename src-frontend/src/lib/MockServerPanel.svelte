<script lang="ts">
  import { workspace } from "./workspaceStore.svelte";
</script>

{#if workspace.info}
  {#if workspace.mockServer?.running}
    <section class="border-b border-neutral-900 bg-neutral-950/70 px-4 py-2">
      <div class="max-h-28 overflow-auto font-mono text-xs leading-5 text-neutral-300">
        {#if workspace.mockServer.logs.length === 0}
          <div class="text-neutral-500">No requests yet.</div>
        {:else}
          {#each workspace.mockServer.logs.slice().reverse() as log}
            <div class="grid grid-cols-[56px_44px_minmax(0,1fr)_minmax(8rem,auto)] gap-2 py-0.5">
              <span class="text-neutral-400">{log.method}</span>
              <span class={log.status >= 500 ? "text-rose-300" : log.status >= 400 ? "text-amber-300" : "text-emerald-300"}>{log.status}</span>
              <span class="truncate text-neutral-300">{log.path}</span>
              <span class="truncate text-neutral-500">{log.matched_rule ?? "(no match)"}</span>
            </div>
          {/each}
        {/if}
      </div>
    </section>
  {/if}
{/if}
