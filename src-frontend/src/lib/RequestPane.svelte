<script lang="ts">
  import type { Compose, Method, ResponseDto } from "./api";
  import { sendRequest } from "./api";
  import KeyValueTable from "./KeyValueTable.svelte";
  import AuthPane from "./AuthPane.svelte";
  import BodyEditor from "./BodyEditor.svelte";

  type Props = {
    loading: boolean;
    onResult: (r: ResponseDto) => void;
    onError: (e: string) => void;
  };

  let { loading = $bindable(), onResult, onError }: Props = $props();

  const methods: Method[] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
  type Tab = "params" | "headers" | "auth" | "body";
  const tabs: Array<{ id: Tab; label: string }> = [
    { id: "params",  label: "Params" },
    { id: "headers", label: "Headers" },
    { id: "auth",    label: "Auth" },
    { id: "body",    label: "Body" },
  ];

  let req: Compose = $state({
    method: "GET",
    url: "https://httpbin.org/get",
    query: [],
    headers: [],
    auth: { kind: "none" },
    body: { kind: "none" },
  });

  let activeTab: Tab = $state("params");

  type Row = { key: string; value: string; enabled: boolean };
  let paramRows: Row[] = $state([]);
  let headerRows: Row[] = $state([]);

  function rowsFrom(pairs: Array<[string, string]>): Row[] {
    return pairs.map(([k, v]) => ({ key: k, value: v, enabled: true }));
  }
  function rowsTo(rows: Row[]): Array<[string, string]> {
    return rows.filter((r) => r.enabled && r.key.length > 0).map((r) => [r.key, r.value]);
  }

  function hasBadge(id: Tab): boolean {
    if (id === "params")  return req.query.length > 0;
    if (id === "headers") return req.headers.length > 0;
    if (id === "auth")    return req.auth.kind !== "none";
    if (id === "body")    return req.body.kind !== "none";
    return false;
  }

  async function send() {
    if (!req.url.trim()) return;
    req.query = rowsTo(paramRows);
    req.headers = rowsTo(headerRows);
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
</script>

<section class="flex flex-col gap-0 rounded-md border border-neutral-800 bg-neutral-900">
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
