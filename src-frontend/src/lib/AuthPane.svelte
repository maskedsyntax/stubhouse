<script lang="ts">
  import type { Auth } from "./api";

  type Props = { auth: Auth };
  let { auth = $bindable() }: Props = $props();

  const types: Array<{ value: Auth["kind"]; label: string }> = [
    { value: "none", label: "None" },
    { value: "bearer", label: "Bearer Token" },
    { value: "basic", label: "Basic Auth" },
    { value: "apikey", label: "API Key" },
  ];

  function setKind(kind: Auth["kind"]) {
    switch (kind) {
      case "none":   auth = { kind: "none" }; break;
      case "bearer": auth = { kind: "bearer", token: "" }; break;
      case "basic":  auth = { kind: "basic", username: "", password: "" }; break;
      case "apikey": auth = { kind: "apikey", in: "header", name: "", value: "" }; break;
    }
  }

  const inputCls =
    "w-full rounded border border-neutral-800 bg-neutral-950 px-2 py-1.5 text-xs outline-none focus:border-indigo-500";
  const labelCls = "flex flex-col gap-1 text-[10px] uppercase tracking-widest text-neutral-500";
</script>

<div class="flex flex-col gap-3 p-3">
  <label class="flex items-center gap-2">
    <span class="text-[10px] uppercase tracking-widest text-neutral-500">Type</span>
    <select
      value={auth.kind}
      onchange={(e) => setKind((e.currentTarget as HTMLSelectElement).value as Auth["kind"])}
      class="rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs outline-none focus:border-indigo-500"
    >
      {#each types as t (t.value)}
        <option value={t.value}>{t.label}</option>
      {/each}
    </select>
  </label>

  {#if auth.kind === "bearer"}
    <label class={labelCls}>
      <span>Token</span>
      <input type="text" bind:value={auth.token} placeholder="eyJ…" class={inputCls} />
    </label>
  {:else if auth.kind === "basic"}
    <div class="grid grid-cols-2 gap-3">
      <label class={labelCls}>
        <span>Username</span>
        <input type="text" bind:value={auth.username} class={inputCls} />
      </label>
      <label class={labelCls}>
        <span>Password</span>
        <input type="password" bind:value={auth.password} class={inputCls} />
      </label>
    </div>
  {:else if auth.kind === "apikey"}
    <div class="grid grid-cols-[120px_1fr_1fr] gap-3">
      <label class={labelCls}>
        <span>Add to</span>
        <select bind:value={auth.in} class={inputCls}>
          <option value="header">Header</option>
          <option value="query">Query param</option>
        </select>
      </label>
      <label class={labelCls}>
        <span>Name</span>
        <input type="text" bind:value={auth.name} placeholder="X-Api-Key" class={inputCls} />
      </label>
      <label class={labelCls}>
        <span>Value</span>
        <input type="text" bind:value={auth.value} class={inputCls} />
      </label>
    </div>
  {:else}
    <p class="text-xs text-neutral-600">No auth will be attached to this request.</p>
  {/if}
</div>
