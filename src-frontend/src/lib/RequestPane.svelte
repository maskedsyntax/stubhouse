<script lang="ts">
  import { sendRequest, type Method, type ResponseDto } from "./api";

  type Props = {
    loading: boolean;
    onResult: (r: ResponseDto) => void;
    onError: (e: string) => void;
  };

  let { loading = $bindable(), onResult, onError }: Props = $props();

  let method: Method = $state("GET");
  let url = $state("https://httpbin.org/get");

  const methods: Method[] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

  async function send() {
    if (!url.trim()) return;
    loading = true;
    try {
      const resp = await sendRequest({ method, url: url.trim(), headers: [], body: null });
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

<section class="flex items-center gap-2 rounded-md border border-neutral-800 bg-neutral-900 p-2">
  <select
    bind:value={method}
    class="rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs font-semibold outline-none focus:border-indigo-500"
  >
    {#each methods as m (m)}
      <option value={m}>{m}</option>
    {/each}
  </select>

  <input
    bind:value={url}
    onkeydown={onKey}
    type="text"
    placeholder="https://api.example.com/users"
    class="flex-1 rounded border border-neutral-700 bg-neutral-950 px-3 py-1.5 outline-none placeholder:text-neutral-600 focus:border-indigo-500"
  />

  <button
    onclick={send}
    disabled={loading || !url.trim()}
    class="rounded bg-indigo-600 px-4 py-1.5 text-xs font-semibold uppercase tracking-wider text-white hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-40"
  >
    {loading ? "…" : "Send"}
  </button>
</section>

<p class="mt-1 text-[10px] text-neutral-600">⌘⏎ to send</p>
