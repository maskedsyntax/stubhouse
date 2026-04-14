<script lang="ts">
  import type { Body } from "./api";
  import KeyValueTable from "./KeyValueTable.svelte";

  type Props = { body: Body };
  let { body = $bindable() }: Props = $props();

  const modes: Array<{ value: Body["kind"]; label: string }> = [
    { value: "none", label: "None" },
    { value: "json", label: "JSON" },
    { value: "text", label: "Text" },
    { value: "form", label: "Form" },
  ];

  function setMode(kind: Body["kind"]) {
    switch (kind) {
      case "none": body = { kind: "none" }; break;
      case "json": body = { kind: "json", text: "" }; break;
      case "text": body = { kind: "text", content_type: "text/plain", text: "" }; break;
      case "form": body = { kind: "form", fields: [] }; break;
    }
  }

  let formRows = $state<Array<{ key: string; value: string; enabled: boolean }>>([]);
  let lastSyncedFromBody = $state<string>("");

  $effect(() => {
    if (body.kind === "form") {
      const canonical = JSON.stringify(body.fields);
      if (canonical !== lastSyncedFromBody) {
        formRows = body.fields.map(([k, v]) => ({ key: k, value: v, enabled: true }));
        lastSyncedFromBody = canonical;
      }
    }
  });

  $effect(() => {
    if (body.kind === "form") {
      const next: Array<[string, string]> = formRows
        .filter((r) => r.enabled && r.key.length > 0)
        .map((r) => [r.key, r.value]);
      const canonical = JSON.stringify(next);
      if (canonical !== lastSyncedFromBody) {
        body = { kind: "form", fields: next };
        lastSyncedFromBody = canonical;
      }
    }
  });
</script>

<div class="flex flex-col gap-2 p-3">
  <div class="flex items-center gap-1">
    {#each modes as m (m.value)}
      <button
        onclick={() => setMode(m.value)}
        class="rounded px-2.5 py-1 text-[10px] uppercase tracking-widest
               {body.kind === m.value
                 ? 'bg-indigo-600 text-white'
                 : 'border border-neutral-800 text-neutral-400 hover:border-indigo-600 hover:text-indigo-400'}"
      >
        {m.label}
      </button>
    {/each}
  </div>

  {#if body.kind === "json"}
    <textarea
      bind:value={body.text}
      spellcheck="false"
      placeholder={'{\n  "name": "Alice"\n}'}
      class="h-48 resize-y rounded border border-neutral-800 bg-neutral-950 p-2 font-mono text-xs outline-none focus:border-indigo-500"
    ></textarea>
  {:else if body.kind === "text"}
    <label class="flex items-center gap-2">
      <span class="text-[10px] uppercase tracking-widest text-neutral-500">Content-Type</span>
      <input
        bind:value={body.content_type}
        class="rounded border border-neutral-800 bg-neutral-950 px-2 py-1 text-xs outline-none focus:border-indigo-500"
      />
    </label>
    <textarea
      bind:value={body.text}
      spellcheck="false"
      class="h-48 resize-y rounded border border-neutral-800 bg-neutral-950 p-2 font-mono text-xs outline-none focus:border-indigo-500"
    ></textarea>
  {:else if body.kind === "form"}
    <KeyValueTable bind:rows={formRows} keyPlaceholder="field" valuePlaceholder="value" />
  {:else}
    <p class="text-xs text-neutral-600">No request body.</p>
  {/if}
</div>
