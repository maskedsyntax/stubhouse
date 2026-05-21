<script lang="ts">
  import type { Compose, HistoryReplay, RequestDefinition, ResponseDto } from "./lib/api";
  import MockServerPanel from "./lib/MockServerPanel.svelte";
  import RequestPane from "./lib/RequestPane.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import TestResultsPanel from "./lib/TestResultsPanel.svelte";
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

  // Stable color per env name — derived from string hash.
  function envColor(name: string): string {
    let h = 0;
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) | 0;
    const palette = [
      "bg-emerald-500",
      "bg-sky-500",
      "bg-amber-500",
      "bg-fuchsia-500",
      "bg-rose-500",
      "bg-violet-500",
      "bg-teal-500",
    ];
    return palette[Math.abs(h) % palette.length];
  }

  async function onEnvChange(e: Event) {
    const value = (e.currentTarget as HTMLSelectElement).value;
    if (value === "") {
      await workspace.deactivate();
    } else {
      await workspace.activate(value);
    }
  }

  async function onScenarioChange(e: Event) {
    const value = (e.currentTarget as HTMLSelectElement).value;
    if (value !== "") await workspace.activateScenario(value);
  }
</script>

<main class="flex h-full">
  <Sidebar onLoad={loadDef} onReplay={loadReplay} />

  <div class="flex flex-1 flex-col">
    <header class="flex items-center justify-between border-b ui-divider bg-neutral-950/80 px-4 py-3">
      <div class="text-[11px] font-semibold uppercase text-neutral-400">
        StubHouse <span class="text-neutral-700">·</span>
        <span class="text-neutral-300">Phase 1 slice C</span>
      </div>

      {#if workspace.info}
        <div class="flex items-center gap-3">
          <label class="flex items-center gap-2 text-xs text-neutral-300">
            <span class="ui-label">scenario</span>
            <select
              class="ui-input py-1 text-xs"
              value={workspace.scenarios.find((scenario) => scenario.active_rules > 0)?.name ?? ""}
              onchange={onScenarioChange}
              disabled={workspace.scenarios.length === 0}
              title={workspace.scenarios.length === 0 ? "No mock scenarios found under collections/*/mocks/" : "Active mock scenario"}
            >
              <option value="">— none —</option>
              {#each workspace.scenarios as scenario (scenario.name)}
                <option value={scenario.name}>
                  {scenario.name} ({scenario.active_rules}/{scenario.rules})
                </option>
              {/each}
            </select>
          </label>

          <label class="flex items-center gap-2 text-xs text-neutral-300">
            <span class="ui-label">env</span>
            {#if workspace.activeEnv}
              <span class="inline-block h-2 w-2 rounded-full {envColor(workspace.activeEnv.name)}"></span>
            {:else}
              <span class="inline-block h-2 w-2 rounded-full bg-neutral-700"></span>
            {/if}
            <select
              class="ui-input py-1 text-xs"
              value={workspace.activeEnv?.name ?? ""}
              onchange={onEnvChange}
              disabled={workspace.envs.length === 0}
              title={workspace.envs.length === 0 ? "No environments — add files under .stubhouse/environments/" : "Active environment"}
            >
              <option value="">— none —</option>
              {#each workspace.envs as env (env.name)}
                <option value={env.name}>{env.name}</option>
              {/each}
            </select>
          </label>
        </div>
      {/if}
    </header>

    <MockServerPanel />
    <TestResultsPanel />

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
