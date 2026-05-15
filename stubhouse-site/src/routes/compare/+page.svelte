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

<section class="container section">
  <p class="eyebrow">Compare</p>
  <h1 class="display-2">Where StubHouse fits.</h1>
  <p class="body-lg prose-width">
    A direct comparison with the tools you already know. Planned items are marked honestly so the table tracks the product as it
    ships.
  </p>

  <div class="compare-table">
    <Comparison />
  </div>
</section>

<style>
  .compare-table {
    margin-top: 40px;
  }

  @media (max-width: 639px) {
    .compare-table {
      margin-top: 28px;
    }
  }
</style>
