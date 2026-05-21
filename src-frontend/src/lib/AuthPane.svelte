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

  const inputCls = "ui-input w-full";
  const labelCls = "flex flex-col gap-1";
</script>

<div class="flex flex-col gap-3 p-3">
  <label class="flex items-center gap-2">
    <span class="ui-label">Type</span>
    <select
      value={auth.kind}
      onchange={(e) => setKind((e.currentTarget as HTMLSelectElement).value as Auth["kind"])}
      class="ui-input"
    >
      {#each types as t (t.value)}
        <option value={t.value}>{t.label}</option>
      {/each}
    </select>
  </label>

  {#if auth.kind === "bearer"}
    <label class={labelCls}>
      <span class="ui-label">Token</span>
      <input type="text" bind:value={auth.token} placeholder="eyJ…" class={inputCls} />
    </label>
  {:else if auth.kind === "basic"}
    <div class="grid grid-cols-2 gap-3">
      <label class={labelCls}>
        <span class="ui-label">Username</span>
        <input type="text" bind:value={auth.username} class={inputCls} />
      </label>
      <label class={labelCls}>
        <span class="ui-label">Password</span>
        <input type="password" bind:value={auth.password} class={inputCls} />
      </label>
    </div>
  {:else if auth.kind === "apikey"}
    <div class="grid grid-cols-[120px_1fr_1fr] gap-3">
      <label class={labelCls}>
        <span class="ui-label">Add to</span>
        <select bind:value={auth.in} class={inputCls}>
          <option value="header">Header</option>
          <option value="query">Query param</option>
        </select>
      </label>
      <label class={labelCls}>
        <span class="ui-label">Name</span>
        <input type="text" bind:value={auth.name} placeholder="X-Api-Key" class={inputCls} />
      </label>
      <label class={labelCls}>
        <span class="ui-label">Value</span>
        <input type="text" bind:value={auth.value} class={inputCls} />
      </label>
    </div>
  {:else}
    <p class="ui-empty">No auth will be attached to this request.</p>
  {/if}
</div>
