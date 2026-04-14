<script lang="ts">
  import type { Compose, HistoryReplay, RequestDefinition, ResponseDto } from "./lib/api";
  import RequestPane from "./lib/RequestPane.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import { workspace } from "./lib/workspaceStore.svelte";

  function defaultReq(): Compose {
    return {
      method: "GET",
      url: "https://httpbin.org/get",
      query: [],
      headers: [],
      auth: { kind: "none" },
      body: { kind: "none" },
    };
  }

  let req: Compose = $state(defaultReq());
  let name = $state("");
  let description = $state("");
  let activeId: string | null = $state(null);

  let response: ResponseDto | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(false);

  function loadDef(def: RequestDefinition, id: string) {
    const { name: n, description: d, ...compose } = def;
    req = compose;
    name = n;
    description = d;
    activeId = id;
    response = null;
    error = null;
  }

  function loadReplay(replay: HistoryReplay) {
    req = replay.request;
    name = "";
    description = "";
    activeId = null;
    workspace.activeId = null;
    response = replay.response;
    error = null;
  }

  function onSendResult(r: ResponseDto) {
    response = r;
    error = null;
    workspace.refreshHistory();
  }
</script>

<main class="flex h-full">
  <Sidebar onLoad={loadDef} onReplay={loadReplay} />

  <div class="flex flex-1 flex-col">
    <header class="border-b border-neutral-800 px-4 py-2 text-xs uppercase tracking-widest text-neutral-500">
      StubHouse <span class="text-neutral-700">·</span> <span class="text-neutral-400">Phase 1 slice C</span>
    </header>

    <div class="flex flex-1 flex-col gap-4 overflow-auto p-4">
      <RequestPane
        bind:loading
        bind:req
        bind:name
        bind:description
        {activeId}
        onResult={onSendResult}
        onError={(e) => { error = e; response = null; }}
        onSaved={(id) => { activeId = id; }}
      />
      <ResponsePanel {response} {error} {loading} />
    </div>
  </div>
</main>
