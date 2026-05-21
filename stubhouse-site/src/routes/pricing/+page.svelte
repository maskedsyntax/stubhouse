<script lang="ts">
  import Check from 'lucide-svelte/icons/check';

  const meta = {
    title: 'Pricing',
    description: 'StubHouse is free and open source. A future cloud tier is sketched honestly, with no invented deadlines.'
  };

  const included = [
    'Desktop API client',
    'CLI automation',
    'File-based workspaces',
    'Postman and OpenAPI import',
    'cURL and OpenAPI export',
    'Local mock server',
    'Scenarios, logs, faults, passthrough, and recording',
    'Rhai assertions and JUnit output'
  ];
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section-tight page-shell">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Pricing</p>
      <h1 class="display-2">Free, forever.</h1>
      <p class="body-lg prose-width">
        The desktop app, CLI, request workflow, environments, imports, exports, history, test runner, and local mock runtime ship
        without a login. No telemetry. No nag screens.
      </p>
    </div>
    <div class="surface-panel price-panel">
      <p class="mono price-panel__label">Open source license</p>
      <p class="price-panel__price">MIT</p>
      <p class="caption price-panel__copy">Use it, fork it, audit it, and run it locally.</p>
    </div>
  </div>

  <div class="feature-grid feature-grid--3 included">
    {#each included as item}
      <div class="feature-card included__item">
        <Check size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>{item}</span>
      </div>
    {/each}
  </div>

  <hr class="divider rule" />

  <h2 class="display-3">Cloud, later.</h2>
  <p class="body-lg prose-width">
    A hosted tier might cover workspace sync, teams, and public mock URLs. It is not on a countdown clock here. If you want a ping
    when that work exists, start a thread on
    <a class="link-inline" href="https://github.com/stubhouse/stubhouse/discussions" target="_blank" rel="noopener noreferrer"
      >GitHub Discussions</a
    >.
  </p>
</section>

<style>
  .price-panel {
    padding: 22px;
  }

  .price-panel__label,
  .price-panel__copy {
    margin: 0;
  }

  .price-panel__price {
    margin: 14px 0 8px;
    font-family: var(--font-display);
    font-size: clamp(56px, 8vw, 92px);
    line-height: 0.95;
    color: var(--text-primary);
  }

  .included {
    margin-top: 42px;
  }

  .included__item {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 60px;
    color: var(--text-secondary);
    font-size: var(--text-body-sm);
  }

  .included__item :global(svg) {
    flex: 0 0 auto;
    color: var(--text-primary);
  }

  .rule {
    margin: 48px 0;
  }
</style>
