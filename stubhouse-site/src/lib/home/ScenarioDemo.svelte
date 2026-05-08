<script lang="ts">
  type Scenario = 'success' | 'not_found' | 'slow_success';

  const payloads: Record<Scenario, { status: number; body: string }> = {
    success: {
      status: 200,
      body: `{
  "id": "{{params.id}}",
  "name": "Alice Nguyen",
  "email": "alice@example.com"
}`
    },
    not_found: {
      status: 404,
      body: `{
  "error": "User not found",
  "code": "USER_NOT_FOUND"
}`
    },
    slow_success: {
      status: 200,
      body: `{
  "id": "{{params.id}}",
  "name": "Alice Nguyen",
  "note": "Delayed response window"
}`
    }
  };

  let active = $state<Scenario>('success');
</script>

<div class="scenario-demo">
  <div class="scenario-demo__ctrl mono" role="group" aria-label="Pick a mock scenario">
    {#each Object.keys(payloads) as s (s)}
      <button
        type="button"
        class="scenario-demo__btn"
        class:scenario-demo__btn--on={active === s}
        onclick={() => (active = s as Scenario)}>{s}</button>
    {/each}
  </div>
  <div class="scenario-demo__out">
    <p class="caption scenario-demo__meta">
      <span class="mono">{payloads[active].status}</span> · synthetic output
    </p>
    <pre class="mono scenario-demo__pre">{payloads[active].body}</pre>
  </div>
</div>

<style>
  .scenario-demo {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-surface);
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.18);
  }

  .scenario-demo__ctrl {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .scenario-demo__btn {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 8px 10px;
    cursor: pointer;
    transition:
      border-color 200ms ease,
      color 200ms ease;
  }

  .scenario-demo__btn:hover,
  .scenario-demo__btn:focus-visible {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .scenario-demo__btn--on {
    color: var(--text-primary);
    border-color: var(--text-primary);
    background: var(--shimmer);
  }

  .scenario-demo__out {
    padding: 12px;
  }

  .scenario-demo__meta {
    margin: 0 0 8px;
  }

  .scenario-demo__pre {
    margin: 0;
    font-size: 12px;
    line-height: 1.45;
    color: var(--text-primary);
    white-space: pre-wrap;
    padding: 10px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--bg-canvas);
  }

  @media (max-width: 639px) {
    .scenario-demo {
      border-radius: var(--radius-sm);
      box-shadow: 0 14px 36px rgba(0, 0, 0, 0.2);
    }

    .scenario-demo__ctrl {
      gap: 6px;
      padding: 10px;
    }

    .scenario-demo__btn {
      flex: 1 1 auto;
      min-width: 0;
      padding: 7px 8px;
      font-size: 11px;
    }

    .scenario-demo__out {
      padding: 10px;
    }

    .scenario-demo__pre {
      overflow-x: auto;
      white-space: pre;
      font-size: 11px;
    }
  }
</style>
