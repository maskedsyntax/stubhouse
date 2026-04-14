<script lang="ts">
  import type { Compose, Method, RequestDefinition, ResponseDto } from "./api";
  import { sendRequest } from "./api";
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

<section class="flex flex-col gap-0 rounded-md border border-neutral-800 bg-neutral-900">
  <div class="flex items-center gap-2 border-b border-neutral-800 px-2 py-2">
    <input
      bind:value={name}
      type="text"
      placeholder="Untitled request"
      class="flex-1 rounded border border-transparent bg-transparent px-2 py-1 text-sm font-semibold text-neutral-100 outline-none hover:border-neutral-800 focus:border-indigo-500"
    />
    <button
      onclick={handleSaveClick}
      class="rounded border border-neutral-700 px-3 py-1.5 text-[11px] uppercase tracking-widest text-neutral-300 hover:border-indigo-500 hover:text-indigo-300"
    >
      Save
    </button>
  </div>

  <div class="flex items-center gap-2 border-b border-neutral-800 p-2">
    <select
      bind:value={req.method}
      class="rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs font-semibold outline-none focus:border-indigo-500"
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
      class="flex-1 rounded border border-neutral-700 bg-neutral-950 px-3 py-1.5 outline-none placeholder:text-neutral-600 focus:border-indigo-500"
    />

    <button
      onclick={send}
      disabled={loading || !req.url.trim()}
      class="rounded bg-indigo-600 px-4 py-1.5 text-xs font-semibold uppercase tracking-wider text-white hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-40"
    >
      {loading ? "…" : "Send"}
    </button>
  </div>

  {#if showSaveDialog}
    <div class="flex flex-col gap-2 border-b border-neutral-800 bg-neutral-950/60 p-3">
      <div class="text-[10px] uppercase tracking-widest text-neutral-500">
        Save request to workspace
      </div>
      <div class="grid grid-cols-[1fr_1fr_auto_auto] items-center gap-2">
        <label class="flex flex-col gap-1">
          <span class="text-[10px] uppercase tracking-widest text-neutral-500">Collection</span>
          <input
            bind:value={saveCollection}
            placeholder="users"
            class="rounded border border-neutral-800 bg-neutral-950 px-2 py-1 text-xs outline-none focus:border-indigo-500"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-[10px] uppercase tracking-widest text-neutral-500">Slug</span>
          <input
            bind:value={saveSlug}
            placeholder="get-users"
            class="rounded border border-neutral-800 bg-neutral-950 px-2 py-1 text-xs outline-none focus:border-indigo-500"
          />
        </label>
        <button
          onclick={confirmSave}
          disabled={!saveCollection.trim() || !saveSlug.trim()}
          class="self-end rounded bg-indigo-600 px-3 py-1.5 text-[11px] uppercase tracking-widest text-white hover:bg-indigo-500 disabled:opacity-40"
        >
          Save
        </button>
        <button
          onclick={() => (showSaveDialog = false)}
          class="self-end rounded border border-neutral-800 px-3 py-1.5 text-[11px] uppercase tracking-widest text-neutral-400 hover:border-neutral-700 hover:text-neutral-200"
        >
          Cancel
        </button>
      </div>
      {#if saveError}
        <div class="text-[11px] text-red-400">{saveError}</div>
      {/if}
    </div>
  {/if}

  <div class="flex items-center gap-1 border-b border-neutral-800 px-2 pt-1">
    {#each tabs as tab (tab.id)}
      <button
        onclick={() => (activeTab = tab.id)}
        class="flex items-center gap-1.5 border-b-2 px-3 py-1.5 text-[11px] uppercase tracking-widest
               {activeTab === tab.id
                 ? 'border-indigo-500 text-neutral-100'
                 : 'border-transparent text-neutral-500 hover:text-neutral-300'}"
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

<p class="mt-1 text-[10px] text-neutral-600">⌘⏎ to send</p>
