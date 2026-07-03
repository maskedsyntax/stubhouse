<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import type { Compose, HistoryReplay, RequestDefinition, ResponseDto } from "./lib/api";
  import { pollSendResult, startSendRequest } from "./lib/api";
  import MockServerPanel from "./lib/MockServerPanel.svelte";
  import RequestPane from "./lib/RequestPane.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import TestResultsPanel from "./lib/TestResultsPanel.svelte";
  import { workspace } from "./lib/workspaceStore.svelte";

  function defaultReq(): Compose {
    return {
      method: "GET",
      url: "https://postman-echo.com/get",
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

  type ResponseState =
    | { mode: "idle"; response: null; error: null }
    | { mode: "loading"; response: null; error: null }
    | { mode: "response"; response: ResponseDto; error: null }
    | { mode: "error"; response: null; error: string };
  let responseState: ResponseState = $state({ mode: "idle", response: null, error: null });
  let mockBind = $state("127.0.0.1");
  let mockPort = $state(4000);
  let mockBusy = $state(false);
  let sendWatchdog: number | null = null;
  let sendToken = 0;

  function loadDef(def: RequestDefinition, id: string) {
    const { name: n, description: d, ...compose } = def;
    req = compose;
    name = n;
    description = d;
    activeId = id;
    responseState = { mode: "idle", response: null, error: null };
  }

  function loadReplay(replay: HistoryReplay) {
    req = replay.request;
    name = "";
    description = "";
    activeId = null;
    workspace.activeId = null;
    responseState = { mode: "response", response: replay.response, error: null };
  }

  async function sendCurrent(nextReq: Compose) {
    const token = ++sendToken;
    if (sendWatchdog !== null) {
      window.clearTimeout(sendWatchdog);
      sendWatchdog = null;
    }
    responseState = { mode: "loading", response: null, error: null };
    sendWatchdog = window.setTimeout(() => {
      if (token === sendToken) {
        sendToken += 1;
        responseState = {
          mode: "error",
          response: null,
          error: "Request timed out after 15 seconds. StubHouse did not receive the response in the UI.",
        };
      }
    }, 15_000);
    try {
      const requestId = await startSendRequest({ ...nextReq, url: nextReq.url.trim() });
      const result = await waitForSendResult(requestId, token);
      if (token !== sendToken) return;
      responseState = { mode: "response", response: result, error: null };
      void workspace.refreshHistory();
    } catch (e) {
      if (token !== sendToken) return;
      responseState = {
        mode: "error",
        response: null,
        error: typeof e === "string" ? e : String(e),
      };
    } finally {
      if (token !== sendToken) return;
      if (sendWatchdog !== null) {
        window.clearTimeout(sendWatchdog);
        sendWatchdog = null;
      }
    }
  }

  async function waitForSendResult(requestId: number, token: number): Promise<ResponseDto> {
    while (token === sendToken) {
      const result = await pollSendResult(requestId);
      if (result.done) {
        if (result.response) return result.response;
        throw new Error(result.error ?? "Request failed without an error message.");
      }
      await new Promise((resolve) => window.setTimeout(resolve, 100));
    }
    throw new Error("Request superseded.");
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

  async function toggleMockServer() {
    mockBusy = true;
    try {
      if (workspace.mockServer?.running) {
        await workspace.stopMock();
      } else {
        await workspace.startMock(mockBind, mockPort);
      }
    } finally {
      mockBusy = false;
    }
  }

  // Delegated drag handler: Tauri's built-in only triggers when event.target itself
  // has `data-tauri-drag-region`. This lets clicks on inert descendants (text, badges)
  // drag the window too, while leaving buttons/inputs/selects untouched.
  function onTitlebarMouseDown(e: MouseEvent) {
    if (e.button !== 0 || e.detail !== 1) return;
    const target = e.target as HTMLElement | null;
    if (!target) return;
    if (target.hasAttribute("data-tauri-drag-region")) return; // Tauri handles direct hits
    if (target.closest('button, input, select, textarea, a, label, [role="button"], [contenteditable="true"]')) return;
    if (!target.closest("[data-tauri-drag-region]")) return;
    e.preventDefault();
    void getCurrentWindow().startDragging();
  }

  onMount(() => {
    document.addEventListener("mousedown", onTitlebarMouseDown);
    const id = window.setInterval(() => {
      if (workspace.mockServer?.running) workspace.refreshMockServer();
    }, 1200);
    return () => {
      document.removeEventListener("mousedown", onTitlebarMouseDown);
      window.clearInterval(id);
    };
  });
</script>

<main class="flex h-full">
  <Sidebar onLoad={loadDef} onReplay={loadReplay} />

  <div class="flex flex-1 flex-col">
    <header data-tauri-drag-region class="native-titlebar flex items-center justify-between gap-4 px-4 pr-4">
      <div data-tauri-drag-region class="flex min-w-0 items-center gap-3">
        <div class="text-sm font-semibold text-neutral-100">StubHouse</div>
        <div class="hidden min-w-0 items-center gap-2 rounded-md border border-neutral-800 bg-neutral-950/70 px-2.5 py-1.5 text-xs text-neutral-400 md:flex">
          <span class="text-neutral-600">workspace</span>
          <span class="max-w-56 truncate text-neutral-200">
            {workspace.info?.manifest.name ?? "No workspace"}
          </span>
        </div>
      </div>

      {#if workspace.info}
        <div class="flex min-w-0 items-center gap-2">
          <div class="hidden items-center gap-2 rounded-md border border-neutral-800 bg-neutral-950/60 px-2.5 py-1.5 text-xs text-neutral-300 lg:flex">
            <span class="h-2 w-2 rounded-full {workspace.mockServer?.running ? 'bg-emerald-500' : 'bg-neutral-700'}"></span>
            <span class="ui-label">mock</span>
            {#if !workspace.mockServer?.running}
              <input
                bind:value={mockPort}
                type="number"
                min="1"
                max="65535"
                aria-label="Mock server port"
                class="w-16 border-0 bg-transparent p-0 font-mono text-xs text-neutral-300 outline-none"
              />
            {/if}
            <span class="font-mono text-neutral-400">
              {workspace.mockServer?.running ? workspace.mockServer.url : "stopped"}
            </span>
            {#if workspace.mockServer?.running}
              <span class="text-neutral-700">·</span>
              <span class="text-neutral-400">{workspace.mockServer.rules} rules</span>
            {/if}
            <button
              onclick={toggleMockServer}
              disabled={mockBusy}
              class="{workspace.mockServer?.running ? 'text-rose-300 hover:text-rose-200' : 'text-emerald-300 hover:text-emerald-200'} disabled:opacity-50"
            >
              {workspace.mockServer?.running ? "Stop" : "Start"}
            </button>
          </div>

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

    <TestResultsPanel />
    <MockServerPanel />

    <div class="flex flex-1 flex-col gap-4 overflow-auto p-4">
      <RequestPane
        loading={responseState.mode === "loading"}
        bind:req
        bind:name
        bind:description
        {activeId}
        onSend={sendCurrent}
        onError={(e) => { responseState = { mode: "error", response: null, error: e }; }}
        onSaved={(id) => { activeId = id; }}
      />
      <ResponsePanel view={responseState} />
    </div>
  </div>
</main>
