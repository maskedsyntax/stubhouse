<script lang="ts">
  import ArrowRight from 'lucide-svelte/icons/arrow-right';
  import Package from 'lucide-svelte/icons/package';
  import Terminal from 'lucide-svelte/icons/terminal';
  import Button from '$lib/components/Button.svelte';

  const meta = {
    title: 'Download',
    description: 'Install StubHouse for macOS, Linux, or Windows — or build from source.'
  };

  let platformOpen = $state(false);

  const platforms = [
    ['macOS', 'Apple Silicon and Intel', '.dmg, Homebrew tap'],
    ['Linux', 'x86_64 and arm64', '.deb, .rpm, AppImage, tar.gz'],
    ['Windows', 'x86_64 and arm64', '.msi, Scoop']
  ];
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section page-shell">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Download</p>
      <h1 class="display-2">Install from source today.</h1>
      <p class="body-lg prose-width">
        Signed installers are not published yet. Until release artifacts exist, the up-to-date path is cloning the repo and building
        the desktop app or CLI from the current source.
      </p>

      <div class="cta">
        <Button href="https://github.com/stubhouse/stubhouse" variant="primary" size="lg" target="_blank" rel="noopener noreferrer">
          {#snippet lead()}<Package size={16} strokeWidth={1.75} aria-hidden="true" />{/snippet}
          {#snippet children()}Open source repo{/snippet}
          {#snippet trailing()}<ArrowRight size={16} strokeWidth={1.75} aria-hidden="true" />{/snippet}
        </Button>
      </div>
    </div>

    <div class="surface-panel install-panel">
      <div class="install-panel__head">
        <Terminal size={18} strokeWidth={1.75} aria-hidden="true" />
        <span class="mono">current install path</span>
      </div>
      <pre class="mono sh-block sh-block--panel">git clone https://github.com/stubhouse/stubhouse
cd stubhouse
cd src-frontend
bun install
bun run tauri dev

cd ..
cargo run -p stubhouse-cli -- --help</pre>
    </div>
  </div>

  <button type="button" class="toggle mono" onclick={() => (platformOpen = !platformOpen)} aria-expanded={platformOpen}>
    Planned installer formats
  </button>

  {#if platformOpen}
    <div class="matrix">
      {#each platforms as row}
        <div class="matrix__row">
          <span class="mono">{row[0]}</span>
          <span>{row[1]}</span>
          <span>{row[2]}</span>
        </div>
      {/each}
    </div>
  {/if}

  <h2 class="display-3 pad-top">Build from source</h2>
  <pre class="mono sh-block">cargo install --path crates/stubhouse-cli
# or
git clone https://github.com/stubhouse/stubhouse && cd stubhouse && cargo build --release</pre>

  <h2 class="display-3 pad-top">Verify</h2>
  <p class="body-lg prose-width">
    SHA256 checksums and signing instructions should ship beside every public artifact. The current source-build path is verified by
    the repository history and local build/test commands rather than by detached release signatures.
  </p>
  <pre class="mono sh-block" aria-label="Verification commands">cargo test -p stubhouse-core
bun run --cwd stubhouse-site check</pre>
</section>

<style>
  .cta {
    margin-top: 24px;
  }

  .toggle {
    margin-top: 24px;
    border: 0;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 4px;
    font-size: 13px;
    padding: 8px 0;
  }

  .matrix {
    margin-top: 16px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
  }

  .matrix__row {
    display: grid;
    grid-template-columns: 120px 1fr 1fr;
    gap: 12px;
    padding: 12px 14px;
  }

  .matrix__row + .matrix__row {
    border-top: 1px solid var(--border-subtle);
  }

  .pad-top {
    margin-top: 56px;
  }

  .install-panel {
    overflow: hidden;
  }

  .install-panel__head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-size: 13px;
  }

  .sh-block {
    margin-top: 12px;
    padding: 16px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    font-size: 13px;
    line-height: 1.55;
    overflow: auto;
  }

  .sh-block--panel {
    margin: 0;
    border: 0;
    border-radius: 0;
    background: transparent;
  }

  @media (max-width: 639px) {
    .matrix__row {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
