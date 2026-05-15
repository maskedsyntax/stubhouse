<script lang="ts">
  type Row = { key: string; value: string; enabled: boolean };

  type Props = {
    rows: Row[];
    keyPlaceholder?: string;
    valuePlaceholder?: string;
  };

  let { rows = $bindable(), keyPlaceholder = "key", valuePlaceholder = "value" }: Props = $props();

  function addRow() {
    rows = [...rows, { key: "", value: "", enabled: true }];
  }

  function removeRow(i: number) {
    rows = rows.filter((_, idx) => idx !== i);
  }
</script>

<div class="flex flex-col">
  <div class="grid grid-cols-[32px_1fr_1fr_32px] gap-1 border-b border-neutral-800 px-2 py-1.5 text-xs font-medium uppercase tracking-wide text-neutral-400">
    <span></span>
    <span>{keyPlaceholder}</span>
    <span>{valuePlaceholder}</span>
    <span></span>
  </div>

  {#each rows as row, i (i)}
    <div class="grid grid-cols-[32px_1fr_1fr_32px] items-center gap-1 border-b border-neutral-900 px-2 py-1.5">
      <input
        type="checkbox"
        bind:checked={row.enabled}
        class="h-3.5 w-3.5 accent-indigo-500"
      />
      <input
        bind:value={row.key}
        type="text"
        placeholder={keyPlaceholder}
        class="rounded border border-transparent bg-neutral-950 px-2 py-1.5 text-sm text-neutral-50 outline-none placeholder:text-neutral-500 focus:border-indigo-500"
      />
      <input
        bind:value={row.value}
        type="text"
        placeholder={valuePlaceholder}
        class="rounded border border-transparent bg-neutral-950 px-2 py-1.5 text-sm text-neutral-50 outline-none placeholder:text-neutral-500 focus:border-indigo-500"
      />
      <button
        onclick={() => removeRow(i)}
        aria-label="remove row"
        class="rounded p-1 text-neutral-500 hover:bg-neutral-800 hover:text-red-300"
      >
        ×
      </button>
    </div>
  {/each}

  <button
    onclick={addRow}
    class="mt-2 self-start rounded border border-neutral-800 px-3 py-1.5 text-xs font-medium uppercase tracking-wide text-neutral-300 hover:border-indigo-600 hover:text-indigo-300"
  >
    + Add row
  </button>
</div>
