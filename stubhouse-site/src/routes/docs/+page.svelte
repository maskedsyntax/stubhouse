<script lang="ts">
  import ArrowRight from 'lucide-svelte/icons/arrow-right';

  const meta = {
    title: 'Documentation',
    description: 'Entry points for getting started, mock servers, CLI automation, and workspace files.'
  };

  const cards = [
    {
      title: 'Getting started',
      href: 'https://github.com/stubhouse/stubhouse#readme',
      copy: 'Clone the repo, run the desktop app, open a workspace, and send the first request.'
    },
    {
      title: 'Mock server',
      href: '/mocks',
      copy: 'YAML rules, scenarios, hot reload, control routes, faults, passthrough, recording, and resources.'
    },
    {
      title: 'CLI reference',
      href: 'https://github.com/stubhouse/stubhouse/blob/main/crates/stubhouse-cli/src/main.rs',
      copy: 'init, validate, list, show, envs, import, export, serve, scenario, and test.'
    }
  ];

  const topics = [
    'Workspace layout',
    'Environment variables',
    'Postman and OpenAPI import',
    'YAML schemas',
    'cURL and OpenAPI export',
    'Headless stubhouse serve',
    'Mock scenarios and live logs',
    'Faults, passthrough, recording, and resource fixtures',
    'Rhai assertions and JUnit XML'
  ];
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section page-shell">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Docs</p>
      <h1 class="display-2">Documentation</h1>
      <p class="body-lg prose-width">
        The public docs are being shaped from the working codebase. These entry points reflect the current CLI, desktop workflow, and
        mock runtime instead of pointing at a separate placeholder host.
      </p>
    </div>
    <pre class="mono-panel" aria-label="CLI commands">{`stubhouse init acme-api
stubhouse import openapi ./openapi.yaml
stubhouse serve --port 4000
stubhouse scenario activate outage
stubhouse test --env ci --junit results.xml`}</pre>
  </div>

  <div class="cards">
    {#each cards as card}
    <a class="card feature-card" href={card.href}>
      <span class="card__top">
        <h2 class="feature-card__title">{card.title}</h2>
        <ArrowRight size={16} strokeWidth={1.75} aria-hidden="true" />
      </span>
      <p class="feature-card__copy">{card.copy}</p>
    </a>
    {/each}
  </div>

  <ul class="mono list surface-panel">
    {#each topics as topic}
      <li>{topic}</li>
    {/each}
  </ul>
</section>

<style>
  .cards {
    margin-top: 40px;
    display: grid;
    gap: 12px;
  }

  @media (min-width: 900px) {
    .cards {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .card {
    display: flex;
    flex-direction: column;
    min-height: 150px;
  }

  .card__top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .card :global(svg) {
    color: var(--text-tertiary);
    flex: 0 0 auto;
  }

  .list {
    margin-top: 32px;
    padding: 18px 18px 18px 34px;
    color: var(--text-secondary);
    line-height: 1.8;
    font-size: 14px;
  }
</style>
