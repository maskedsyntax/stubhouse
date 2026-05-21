<script lang="ts">
  import type { Compose, Method, RequestDefinition, ResponseDto } from "./api";
  import { exportCurl, sendRequest } from "./api";
  import { workspace } from "./workspaceStore.svelte";
  import KeyValueTable from "./KeyValueTable.svelte";
  import AuthPane from "./AuthPane.svelte";
  import BodyEditor from "./BodyEditor.svelte";

  type Props = {
    loading: boolean;
    req: Compose;
    name: string;
    description: string;
    activeId: string | null;
    onResult: (r: ResponseDto) => void;
    onError: (e: string) => void;
    onSaved: (id: string) => void;
  };

  let {
    loading = $bindable(),
    req = $bindable(),
    name = $bindable(),
    description = $bindable(),
    activeId,
    onResult,
    onError,
    onSaved,
  }: Props = $props();

  const methods: Method[] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

  // Mirror of stubhouse_core::interpolate_string for {{key}} preview only.
  // Builtins ($timestamp, $randomUUID, $env.*) are intentionally left as-is.
  function interpolate(s: string, vars: Record<string, string>): string {
    return s.replace(/\{\{\s*([^}]+?)\s*\}\}/g, (whole, key) =>
      key.startsWith("$") || !(key in vars) ? whole : vars[key],
    );
  }

  function variableNames(s: string): string[] {
    return Array.from(s.matchAll(/\{\{\s*([^}]+?)\s*\}\}/g), (match) => match[1].trim());
  }

  const unresolvedUrlVariables = $derived.by(() => {
    const vars = workspace.activeEnv?.variables ?? {};
    return variableNames(req.url)
      .filter((key) => !key.startsWith("$") && !(key in vars))
      .filter((key, index, keys) => keys.indexOf(key) === index);
  });

  const resolvedUrl = $derived.by(() => {
    const vars = workspace.activeEnv?.variables ?? {};
    if (!req.url.includes("{{")) return null;
    const out = interpolate(req.url, vars);
    return out === req.url ? null : out;
  });

  const canSend = $derived(
    !loading && req.url.trim().length > 0 && unresolvedUrlVariables.length === 0,
  );

  type Tab = "params" | "headers" | "auth" | "body";
  const tabs: Array<{ id: Tab; label: string }> = [
    { id: "params",  label: "Params" },
    { id: "headers", label: "Headers" },
    { id: "auth",    label: "Auth" },
    { id: "body",    label: "Body" },
  ];
  let activeTab: Tab = $state("params");

  type Row = { key: string; value: string; enabled: boolean };
  let paramRows: Row[] = $state([]);
  let headerRows: Row[] = $state([]);

  let lastSyncedParams = $state("");
  let lastSyncedHeaders = $state("");
  let lastPushedParams = $state("");
  let lastPushedHeaders = $state("");

  $effect(() => {
    const canonical = JSON.stringify(req.query);
    if (canonical !== lastSyncedParams) {
      paramRows = req.query.map(([k, v]) => ({ key: k, value: v, enabled: true }));
      lastSyncedParams = canonical;
      lastPushedParams = canonical;
    }
  });
  $effect(() => {
    const canonical = JSON.stringify(req.headers);
    if (canonical !== lastSyncedHeaders) {
      headerRows = req.headers.map(([k, v]) => ({ key: k, value: v, enabled: true }));
      lastSyncedHeaders = canonical;
      lastPushedHeaders = canonical;
    }
  });
  $effect(() => {
    const next: Array<[string, string]> = paramRows
      .filter((r) => r.enabled && r.key.length > 0)
      .map((r) => [r.key, r.value]);
    const canonical = JSON.stringify(next);
    if (canonical !== lastPushedParams) {
      req.query = next;
      lastPushedParams = canonical;
      lastSyncedParams = canonical;
    }
  });
  $effect(() => {
    const next: Array<[string, string]> = headerRows
      .filter((r) => r.enabled && r.key.length > 0)
      .map((r) => [r.key, r.value]);
    const canonical = JSON.stringify(next);
    if (canonical !== lastPushedHeaders) {
      req.headers = next;
      lastPushedHeaders = canonical;
      lastSyncedHeaders = canonical;
    }
  });

  function hasBadge(id: Tab): boolean {
    if (id === "params")  return req.query.length > 0;
    if (id === "headers") return req.headers.length > 0;
    if (id === "auth")    return req.auth.kind !== "none";
    if (id === "body")    return req.body.kind !== "none";
    return false;
  }

  async function send() {
    if (!req.url.trim()) return;
    if (unresolvedUrlVariables.length > 0) {
      onError(
        workspace.activeEnv
          ? `Missing variable${unresolvedUrlVariables.length === 1 ? "" : "s"}: ${unresolvedUrlVariables.join(", ")}`
          : `Select an environment to resolve: ${unresolvedUrlVariables.join(", ")}`,
      );
      return;
    }
    loading = true;
    try {
      const resp = await sendRequest({ ...req, url: req.url.trim() });
      onResult(resp);
    } catch (e) {
      onError(typeof e === "string" ? e : String(e));
    } finally {
      loading = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") send();
  }

  let showSaveDialog = $state(false);
  let saveCollection = $state("");
  let saveSlug = $state("");
  let saveError = $state<string | null>(null);

  let curlFlash = $state<"idle" | "copied" | "error">("idle");

  async function copyCurl() {
    if (unresolvedUrlVariables.length > 0) {
      curlFlash = "error";
      onError(
        workspace.activeEnv
          ? `Missing variable${unresolvedUrlVariables.length === 1 ? "" : "s"}: ${unresolvedUrlVariables.join(", ")}`
          : `Select an environment to resolve: ${unresolvedUrlVariables.join(", ")}`,
      );
      setTimeout(() => (curlFlash = "idle"), 1500);
      return;
    }
    try {
      const snippet = await exportCurl({ ...req, url: req.url.trim() });
      await navigator.clipboard.writeText(snippet);
      curlFlash = "copied";
    } catch (e) {
      curlFlash = "error";
      onError(typeof e === "string" ? e : String(e));
    }
    setTimeout(() => (curlFlash = "idle"), 1500);
  }

  function slugify(s: string): string {
    return s
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 80);
  }

  async function handleSaveClick() {
    if (!workspace.info) {
      saveError = "Open a workspace first (sidebar → Open).";
      showSaveDialog = true;
      return;
    }
    if (activeId) {
      const parts = activeId.split("/");
      const col = parts[1] ?? "default";
      const file = parts.at(-1) ?? "";
      const slug = file.replace(/\.ya?ml$/, "");
      const id = await workspace.save(col, slug, name || "Untitled", description, req);
      if (id) onSaved(id);
      return;
    }
    saveCollection = "default";
    saveSlug = slugify(name) || "new-request";
    saveError = null;
    showSaveDialog = true;
  }

  async function confirmSave() {
    if (!workspace.info) return;
    const id = await workspace.save(
      saveCollection,
      saveSlug,
      name || "Untitled",
      description,
      req,
    );
    if (id) {
      showSaveDialog = false;
      onSaved(id);
    } else {
      saveError = workspace.error;
    }
  }
</script>

<section class="ui-panel flex flex-col gap-0">
  <div class="flex items-center gap-2 border-b ui-divider px-2 py-2">
    <input
      bind:value={name}
      type="text"
      placeholder="Untitled request"
      class="flex-1 rounded-md border border-transparent bg-transparent px-2 py-1 text-[15px] font-semibold text-neutral-50 outline-none transition-colors hover:border-neutral-800 focus:border-indigo-500"
    />
    <button
      onclick={copyCurl}
      disabled={!req.url.trim()}
      title="Copy as cURL"
      class="ui-button uppercase
             {curlFlash === 'copied' ? 'text-emerald-300' : curlFlash === 'error' ? 'text-red-300' : 'text-neutral-200'}"
    >
      {curlFlash === "copied" ? "Copied" : curlFlash === "error" ? "Failed" : "cURL"}
    </button>
    <button
      onclick={handleSaveClick}
      class="ui-button uppercase"
    >
      Save
    </button>
  </div>

  <div class="flex items-center gap-2 border-b ui-divider p-2">
    <select
      bind:value={req.method}
      class="ui-input w-24 font-semibold"
    >
      {#each methods as m (m)}
        <option value={m}>{m}</option>
      {/each}
    </select>

    <input
      bind:value={req.url}
      onkeydown={onKey}
      type="text"
      placeholder="https://api.example.com/users"
      class="ui-input flex-1 px-3 text-[15px]"
    />

    <button
      onclick={send}
      disabled={!canSend}
      class="ui-button-primary"
    >
      {loading ? "…" : "Send"}
    </button>
  </div>

  {#if unresolvedUrlVariables.length > 0}
    <div class="border-b border-amber-900/50 bg-amber-950/30 px-3 py-1.5 text-xs text-amber-200">
      {#if workspace.activeEnv}
        Missing variable{unresolvedUrlVariables.length === 1 ? "" : "s"}:
      {:else}
        Select an environment to resolve:
      {/if}
      <span class="font-mono">{unresolvedUrlVariables.join(", ")}</span>
    </div>
  {/if}

  {#if resolvedUrl}
    <div class="border-b ui-divider bg-neutral-950/50 px-3 py-1.5 text-xs text-neutral-400">
      <span class="uppercase tracking-wide text-neutral-500">→</span>
      <span class="ml-1 font-mono text-neutral-300">{resolvedUrl}</span>
    </div>
  {/if}

  {#if showSaveDialog}
    <div class="flex flex-col gap-2 border-b ui-divider bg-neutral-950/60 p-3">
      <div class="ui-label">
        Save request to workspace
      </div>
      <div class="grid grid-cols-[1fr_1fr_auto_auto] items-center gap-2">
        <label class="flex flex-col gap-1">
          <span class="ui-label">Collection</span>
          <input
            bind:value={saveCollection}
            placeholder="users"
            class="ui-input"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="ui-label">Slug</span>
          <input
            bind:value={saveSlug}
            placeholder="get-users"
            class="ui-input"
          />
        </label>
        <button
          onclick={confirmSave}
          disabled={!saveCollection.trim() || !saveSlug.trim()}
          class="ui-button-primary self-end px-3"
        >
          Save
        </button>
        <button
          onclick={() => (showSaveDialog = false)}
          class="ui-button self-end uppercase"
        >
          Cancel
        </button>
      </div>
      {#if saveError}
        <div class="text-xs text-red-300">{saveError}</div>
      {/if}
    </div>
  {/if}

  <div class="flex items-center gap-1 border-b ui-divider px-2 pt-1">
    {#each tabs as tab (tab.id)}
      <button
        onclick={() => (activeTab = tab.id)}
        class="ui-tab
               {activeTab === tab.id
                 ? 'ui-tab-active'
                 : ''}"
      >
        {tab.label}
        {#if hasBadge(tab.id)}
          <span class="h-1.5 w-1.5 rounded-full bg-indigo-500"></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="min-h-[160px]">
    {#if activeTab === "params"}
      <div class="p-2">
        <KeyValueTable bind:rows={paramRows} keyPlaceholder="param" valuePlaceholder="value" />
      </div>
    {:else if activeTab === "headers"}
      <div class="p-2">
        <KeyValueTable bind:rows={headerRows} keyPlaceholder="header" valuePlaceholder="value" />
      </div>
    {:else if activeTab === "auth"}
      <AuthPane bind:auth={req.auth} />
    {:else if activeTab === "body"}
      <BodyEditor bind:body={req.body} />
    {/if}
  </div>
</section>

<p class="mt-1 text-xs text-neutral-500">⌘⏎ to send</p>
