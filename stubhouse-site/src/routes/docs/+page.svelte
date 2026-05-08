<script lang="ts">
  const meta = {
    title: 'Documentation',
    description: 'Entry points for getting started, mock servers, and API reference — hosted on docs.stubhouse.dev.'
  };
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section">
  <p class="eyebrow">Docs</p>
  <h1 class="display-2">Documentation</h1>
  <p class="body-lg prose-width">
    Primary docs will live at <a class="link-inline" href="https://docs.stubhouse.dev">docs.stubhouse.dev</a>. This gateway stays on the
    marketing domain so we can move engines without breaking navigation.
  </p>

  <div class="cards">
    <a class="card link-inline" href="https://docs.stubhouse.dev/getting-started">
      <h2 class="display-3">Getting started</h2>
      <p class="caption">Install, open a workspace, send your first request.</p>
    </a>
    <a class="card link-inline" href="https://docs.stubhouse.dev/mocks">
      <h2 class="display-3">Mock server guide</h2>
      <p class="caption">Rules, scenarios, state, recording, faults, control API.</p>
    </a>
    <a class="card link-inline" href="https://docs.stubhouse.dev/cli">
      <h2 class="display-3">CLI reference</h2>
      <p class="caption">`serve`, `test`, `scenario`, CI snippets.</p>
    </a>
  </div>

  <label class="visually-hidden" for="doc-search">Search the docs</label>
  <input id="doc-search" class="search mono" type="search" placeholder="Search (wired to docs host next)" disabled />

  <ul class="mono list">
    <li>Workspace layout</li>
    <li>Environment variables</li>
    <li>Import formats</li>
    <li>YAML schemas</li>
    <li>Testing with JUnit output</li>
    <li>Headless `stubhouse serve`</li>
    <li>Passthrough + recording</li>
    <li>Plugin manifests</li>
    <li>Troubleshooting TLS</li>
  </ul>
</section>

<style>
  .cards {
    margin-top: 40px;
    display: grid;
    gap: 16px;
  }

  @media (min-width: 900px) {
    .cards {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .card {
    display: block;
    padding: 20px;
    border: 1px solid var(--border-default);
    background: var(--bg-surface);
    text-decoration: none;
    color: inherit;
    transition: border-color 200ms ease;
  }

  .card:hover,
  .card:focus-visible {
    border-color: var(--border-strong);
  }

  .card .caption {
    margin: 8px 0 0;
  }

  .search {
    margin-top: 40px;
    width: 100%;
    max-width: 480px;
    min-height: 48px;
    padding: 0 12px;
    border: 1px solid var(--border-default);
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .list {
    margin-top: 32px;
    padding-left: 18px;
    color: var(--text-secondary);
    line-height: 1.8;
    font-size: 14px;
  }
</style>
