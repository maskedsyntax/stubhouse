<script lang="ts">
  const meta = {
    title: 'Changelog',
    description: 'Release notes for StubHouse — newest first.'
  };

  const releases = [
    {
      version: '0.2.0-dev',
      date: '2026-05-21',
      summary: 'Phase 1 is complete and the mock server/test-runner slice is now visible in both CLI and desktop workflows.',
      sections: {
        New: [
          'Environment files and variable interpolation wired through the app',
          'Environment switcher and resolved URL preview',
          'Postman Collection v2.1 and OpenAPI 3 import',
          'Copy/export as cURL and OpenAPI YAML export',
          'CLI commands for init, validate, list, show, envs, import, export, serve, scenario, and test',
          'Embedded mock server module with YAML rule parsing, priority route matching, hot reload, and live request logs',
          'Named mock scenarios with activation from the CLI and desktop app',
          'Fault injection, passthrough, recording config, and stateful mock resources',
          'Rhai response assertions with optional JUnit XML output'
        ],
        Improved: [
          'Workspace layout now keeps environments beside collections at the workspace root',
          'Desktop app now exposes mock server controls, scenario selection, and test results'
        ],
        Fixed: [],
        Breaking: []
      },
      href: 'https://github.com/stubhouse/stubhouse/releases'
    }
  ];
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section page-shell">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Changelog</p>
      <h1 class="display-2">Releases</h1>
      <p class="body-lg prose-width">
        This page tracks the working dev line until tagged GitHub Releases are stable. It is curated from the current CLI, core, and
        desktop app surface.
      </p>
    </div>
    <div class="surface-panel release-summary">
      <p class="mono release-summary__label">Current public state</p>
      <strong>0.2.0-dev</strong>
      <span>Source builds only. Release artifacts are still pending.</span>
    </div>
  </div>

  {#each releases as r}
    <article class="release">
      <div class="release__head">
        <h2 class="mono release__ver">{r.version}</h2>
        <time class="caption" datetime={r.date}>{r.date}</time>
      </div>
      <p class="body-lg">{r.summary}</p>
      {#each Object.entries(r.sections) as [k, items] (k)}
        {#if items.length}
          <h3 class="release__sub mono">{k}</h3>
          <ul>
            {#each items as line}
              <li>{line}</li>
            {/each}
          </ul>
        {/if}
      {/each}
      <p class="caption"><a class="link-inline" href={r.href}>View on GitHub →</a></p>
    </article>
  {/each}
</section>

<style>
  .release {
    margin-top: 48px;
    padding: 24px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    box-shadow: inset 0 1px 0 var(--border-subtle);
  }

  .release-summary {
    padding: 22px;
  }

  .release-summary__label {
    margin: 0 0 14px;
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .release-summary strong {
    display: block;
    font-family: var(--font-display);
    font-size: 42px;
    line-height: 1;
    font-weight: 400;
    color: var(--text-primary);
  }

  .release-summary span {
    display: block;
    margin-top: 12px;
    color: var(--text-secondary);
    font-size: var(--text-body-sm);
    line-height: 1.5;
  }

  .release__head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 12px 24px;
    margin-bottom: 12px;
  }

  .release__ver {
    margin: 0;
    font-size: 28px;
    font-weight: 500;
    letter-spacing: 0;
  }

  .release__sub {
    margin: 24px 0 8px;
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .release ul {
    margin: 0;
    padding-left: 18px;
    color: var(--text-secondary);
  }

  .release li {
    margin: 4px 0;
  }
</style>
