<script lang="ts">
  const meta = {
    title: 'Pricing',
    description: 'StubHouse is free and open source. A future cloud tier is sketched honestly, with no invented deadlines.'
  };
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section-tight">
  <p class="eyebrow">Pricing</p>
  <h1 class="display-2">Free, forever.</h1>
  <p class="body-lg prose-width">
    The desktop app, CLI, request workflow, environments, imports, cURL export, history, and local mock runtime ship without a login.
    No telemetry. No nag screens.
  </p>

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
  .rule {
    margin: 48px 0;
  }
</style>
