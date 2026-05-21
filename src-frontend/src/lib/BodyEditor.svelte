<script lang="ts">
  import type { Body } from "./api";
  import KeyValueTable from "./KeyValueTable.svelte";
  import SyntaxBlock from "./SyntaxBlock.svelte";

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
  let jsonOverlay: HTMLDivElement | null = $state(null);

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

  function syncJsonScroll(e: Event) {
    if (jsonOverlay) {
      const textarea = e.currentTarget as HTMLTextAreaElement;
      jsonOverlay.scrollTop = textarea.scrollTop;
      jsonOverlay.scrollLeft = textarea.scrollLeft;
    }
  }
</script>

<div class="flex flex-col gap-2 p-3">
  <div class="flex items-center gap-1">
    {#each modes as m (m.value)}
      <button
        onclick={() => setMode(m.value)}
        class="rounded-md px-2.5 py-1.5 text-xs font-medium uppercase transition-colors
               {body.kind === m.value
                 ? 'bg-indigo-600 text-white'
                 : 'border border-neutral-800 text-neutral-300 hover:border-indigo-600 hover:text-indigo-300'}"
      >
        {m.label}
      </button>
    {/each}
  </div>

  {#if body.kind === "json"}
    <div class="relative h-48 overflow-hidden rounded-md border border-neutral-800 bg-neutral-950 focus-within:border-indigo-500">
      <div bind:this={jsonOverlay} class="pointer-events-none absolute inset-0 overflow-hidden">
        <SyntaxBlock
          code={body.text || '{\n  \"name\": \"Alice\"\n}'}
          language="json"
          class="whitespace-pre p-3 font-mono text-sm leading-6"
        />
      </div>
      <textarea
        bind:value={body.text}
        onscroll={syncJsonScroll}
        spellcheck="false"
        placeholder={'{\n  "name": "Alice"\n}'}
        class="absolute inset-0 h-full w-full resize-none overflow-auto border-0 bg-transparent p-3 font-mono text-sm leading-6 text-transparent caret-indigo-200 outline-none placeholder:text-neutral-600"
      ></textarea>
    </div>
  {:else if body.kind === "text"}
    <label class="flex items-center gap-2">
      <span class="ui-label">Content-Type</span>
      <input
        bind:value={body.content_type}
        class="ui-input"
      />
    </label>
    <textarea
      bind:value={body.text}
      spellcheck="false"
      class="ui-input h-48 resize-y p-3 font-mono leading-6"
    ></textarea>
  {:else if body.kind === "form"}
    <KeyValueTable bind:rows={formRows} keyPlaceholder="field" valuePlaceholder="value" />
  {:else}
    <p class="ui-empty">No request body.</p>
  {/if}
</div>
