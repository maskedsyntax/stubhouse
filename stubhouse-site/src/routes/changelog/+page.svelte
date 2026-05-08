<script lang="ts">
  const meta = {
    title: 'Changelog',
    description: 'Release notes for StubHouse — newest first.'
  };

  const releases = [
    {
      version: '0.1.0',
      date: '2026-05-01',
      summary: 'Foundational workspace parsing and a minimal request path through the Rust core.',
      sections: {
        New: ['Workspace manifest', 'SQLite request history'],
        Improved: ['Request compose draft UI'],
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

<section class="container section">
  <p class="eyebrow">Changelog</p>
  <h1 class="display-2">Releases</h1>
  <p class="body-lg prose-width">
    This page is hand-curated today and can be generated from GitHub Releases once tags are stable.
  </p>

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
    padding-top: 32px;
    border-top: 1px solid var(--border-subtle);
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
    letter-spacing: -0.02em;
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
