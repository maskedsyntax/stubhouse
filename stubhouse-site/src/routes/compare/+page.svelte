<script lang="ts">
  import Comparison from '$lib/components/Comparison.svelte';

  const meta = {
    title: 'Compare',
    description: 'Compare StubHouse with Postman, Insomnia, Bruno, and Yaak across local-first workflows and mock server features.'
  };
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section page-shell">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Compare</p>
      <h1 class="display-2">Where StubHouse fits.</h1>
      <p class="body-lg prose-width">
        A direct comparison with the tools you already know. Planned items are marked honestly so the table tracks the product as it
        ships.
      </p>
    </div>
    <div class="surface-panel compare-note">
      <p class="mono">Comparison stance</p>
      <span>Local files, native app shell, embedded mocks, and no cloud account in the critical path.</span>
    </div>
  </div>

  <div class="compare-table">
    <Comparison />
  </div>
</section>

<style>
  .compare-table {
    margin-top: 40px;
  }

  .compare-note {
    padding: 22px;
  }

  .compare-note p {
    margin: 0 0 12px;
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .compare-note span {
    color: var(--text-secondary);
    font-size: var(--text-body-sm);
    line-height: 1.55;
  }

  @media (max-width: 639px) {
    .compare-table {
      margin-top: 28px;
    }
  }
</style>
