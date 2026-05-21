<script lang="ts">
  import { workspace } from "./workspaceStore.svelte";

  const total = $derived(workspace.testRun?.assertions.length ?? 0);
  const failed = $derived(workspace.testRun?.assertions.filter((test) => !test.passed).length ?? 0);
  const passed = $derived(total - failed);
</script>

<section class="border-b border-neutral-800 bg-neutral-950 px-4 py-3">
  <div class="flex items-center justify-between gap-3">
    <div class="flex items-center gap-3 text-xs">
      <span class="font-medium uppercase tracking-wide text-neutral-500">tests</span>
      <span class="text-neutral-300">{passed}/{total} passed</span>
      {#if failed > 0}
        <span class="text-rose-400">{failed} failed</span>
      {/if}
    </div>
    <button
      class="rounded border border-neutral-700 px-3 py-1.5 text-xs font-medium text-neutral-100 hover:border-neutral-500 disabled:cursor-not-allowed disabled:opacity-50"
      disabled={!workspace.info || workspace.testsRunning}
      onclick={() => workspace.runWorkspaceTests()}
    >
      {workspace.testsRunning ? "Running..." : "Run tests"}
    </button>
  </div>

  {#if workspace.testRun && workspace.testRun.assertions.length > 0}
    <div class="mt-3 max-h-36 overflow-auto rounded border border-neutral-800">
      {#each workspace.testRun.assertions as test}
        <div class="grid grid-cols-[56px_1fr] gap-2 border-b border-neutral-900 px-3 py-2 text-xs last:border-b-0">
          <span class={test.passed ? "text-emerald-400" : "text-rose-400"}>{test.passed ? "ok" : "fail"}</span>
          <div>
            <div class="text-neutral-200">{test.name}</div>
            <div class="text-neutral-500">{test.request_id}</div>
            {#if test.message}
              <div class="mt-1 text-rose-300">{test.message}</div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>
