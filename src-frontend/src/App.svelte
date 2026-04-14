<script lang="ts">
  import RequestPane from "./lib/RequestPane.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import type { ResponseDto } from "./lib/api";

  let response: ResponseDto | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(false);
</script>

<main class="flex h-full flex-col">
  <header class="border-b border-neutral-800 px-4 py-2 text-xs uppercase tracking-widest text-neutral-500">
    StubHouse <span class="text-neutral-700">·</span> <span class="text-neutral-400">vertical slice</span>
  </header>

  <div class="flex flex-1 flex-col gap-4 overflow-auto p-4">
    <RequestPane
      bind:loading
      onResult={(r) => { response = r; error = null; }}
      onError={(e) => { error = e; response = null; }}
    />
    <ResponsePanel {response} {error} {loading} />
  </div>
</main>
