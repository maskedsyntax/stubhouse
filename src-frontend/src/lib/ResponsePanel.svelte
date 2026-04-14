<script lang="ts">
  import type { ResponseDto } from "./api";

  type Props = {
    response: ResponseDto | null;
    error: string | null;
    loading: boolean;
  };

  let { response, error, loading }: Props = $props();

  type Tab = "body" | "headers";
  let activeTab: Tab = $state("body");

  function statusTone(status: number): string {
    if (status >= 500) return "bg-red-900/60 text-red-200 border-red-700";
    if (status >= 400) return "bg-amber-900/60 text-amber-200 border-amber-700";
    if (status >= 300) return "bg-sky-900/60 text-sky-200 border-sky-700";
    if (status >= 200) return "bg-emerald-900/60 text-emerald-200 border-emerald-700";
    return "bg-neutral-800 text-neutral-300 border-neutral-700";
  }

  function fmtBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / 1024 / 1024).toFixed(2)} MB`;
  }

  function tryPrettyJson(body: string): string {
    try {
      return JSON.stringify(JSON.parse(body), null, 2);
    } catch {
      return body;
    }
  }
</script>

<section class="flex flex-1 flex-col overflow-hidden rounded-md border border-neutral-800 bg-neutral-900">
  {#if loading}
    <div class="flex flex-1 items-center justify-center text-neutral-500">sending…</div>
  {:else if error}
    <div class="flex flex-1 flex-col gap-2 p-4">
      <div class="text-xs uppercase tracking-wider text-red-400">Error</div>
      <pre class="whitespace-pre-wrap text-red-300">{error}</pre>
    </div>
  {:else if response}
    <header class="flex items-center gap-3 border-b border-neutral-800 px-3 py-2 text-xs">
      <span class="rounded border px-2 py-0.5 font-semibold {statusTone(response.status)}">
        {response.status}
      </span>
      <span class="text-neutral-400">{response.elapsed_ms} ms</span>
      <span class="text-neutral-700">·</span>
      <span class="text-neutral-400">{fmtBytes(response.size_bytes)}</span>
      <span class="ml-auto text-neutral-600">{response.headers.length} headers</span>
    </header>

    <div class="flex items-center gap-1 border-b border-neutral-800 px-2 pt-1">
      {#each [{ id: "body" as Tab, label: "Body" }, { id: "headers" as Tab, label: "Headers" }] as t (t.id)}
        <button
          onclick={() => (activeTab = t.id)}
          class="border-b-2 px-3 py-1.5 text-[11px] uppercase tracking-widest
                 {activeTab === t.id
                   ? 'border-indigo-500 text-neutral-100'
                   : 'border-transparent text-neutral-500 hover:text-neutral-300'}"
        >
          {t.label}
        </button>
      {/each}
    </div>

    {#if activeTab === "body"}
      <pre class="flex-1 overflow-auto p-3 text-xs leading-relaxed text-neutral-100">{tryPrettyJson(response.body)}</pre>
    {:else}
      <div class="flex-1 overflow-auto">
        <table class="w-full text-xs">
          <tbody>
            {#each response.headers as [k, v], i (i)}
              <tr class="border-b border-neutral-900">
                <td class="w-1/3 px-3 py-1.5 align-top text-neutral-400">{k}</td>
                <td class="px-3 py-1.5 align-top text-neutral-100 break-all">{v}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else}
    <div class="flex flex-1 items-center justify-center text-neutral-600">
      no response yet
    </div>
  {/if}
</section>
