<section class="container press page-shell">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Press</p>
      <h1 class="display-2">Brand kit</h1>
      <p class="body-lg prose-width">
        Use the shipped marks in this site until a full press archive exists. The current public story is source-build dev software,
        not a packaged v1 release.
      </p>
    </div>
    <div class="surface-panel press-mark">
      <img src="/stubhouse-logo-rounded-512.png" width="160" height="160" alt="" />
      <span class="mono">StubHouse mark</span>
    </div>
  </div>
  <ul class="mono list surface-panel">
    <li><a class="link-inline" href="/stubhouse-logo.png">stubhouse-logo.png</a><span>primary 3D mark</span></li>
    <li><a class="link-inline" href="/stubhouse-logo-rounded.png">stubhouse-logo-rounded.png</a><span>rounded-corner site mark</span></li>
    <li><a class="link-inline" href="/mark.svg">mark.svg</a><span>flat vector fallback</span></li>
    <li><span>Lockups and editorial PDF</span><span>planned for packaged release</span></li>
  </ul>
</section>

<svelte:head>
  <title>Press — StubHouse</title>
  <meta name="description" content="StubHouse press resources — logos and brand guidelines." />
</svelte:head>

<style>
  .press {
    padding-block: 80px;
  }

  .press-mark {
    display: grid;
    place-items: center;
    gap: 14px;
    padding: 28px;
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .press-mark img {
    border-radius: 32px;
    box-shadow: 0 22px 70px rgba(0, 0, 0, 0.24);
  }

  .list {
    margin-top: 32px;
    padding: 0;
    list-style: none;
    color: var(--text-secondary);
    overflow: hidden;
  }

  .list li {
    display: grid;
    grid-template-columns: minmax(0, 280px) minmax(0, 1fr);
    gap: 16px;
    padding: 14px 16px;
  }

  .list li + li {
    border-top: 1px solid var(--border-subtle);
  }

  @media (max-width: 639px) {
    .list li {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
