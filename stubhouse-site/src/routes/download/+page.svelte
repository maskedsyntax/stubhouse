<script lang="ts">
  import Button from '$lib/components/Button.svelte';

  const meta = {
    title: 'Download',
    description: 'Install StubHouse for macOS, Linux, or Windows — or build from source.'
  };

  let platformOpen = $state(false);
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section">
  <p class="eyebrow">Download</p>
  <h1 class="display-2">Install the binary.</h1>
  <p class="body-lg prose-width">
    Detected defaults will arrive with releases. For now, this page documents the formats we ship and what to verify once artifacts are
    published on GitHub Releases.
  </p>

  <div class="cta">
    <Button href="https://github.com/stubhouse/stubhouse/releases" variant="primary" size="lg" target="_blank" rel="noopener noreferrer">
      {#snippet children()}Open releases{/snippet}
    </Button>
  </div>

  <button type="button" class="toggle mono" onclick={() => (platformOpen = !platformOpen)} aria-expanded={platformOpen}>
    Choose another platform
  </button>

  {#if platformOpen}
    <div class="matrix mono">
      <p>macOS · Apple Silicon / Intel · .dmg · Homebrew tap</p>
      <p>Linux · x86_64 / arm64 · .deb · .rpm · .AppImage · .tar.gz</p>
      <p>Windows · x86_64 / arm64 · .msi · Scoop</p>
    </div>
  {/if}

  <h2 class="display-3 pad-top">Build from source</h2>
  <pre class="mono sh-block">cargo install --path crates/stubhouse-cli
# or
git clone https://github.com/stubhouse/stubhouse && cd stubhouse && cargo build --release</pre>

  <h2 class="display-3 pad-top">Verify</h2>
  <p class="body-lg prose-width">
    SHA256 checksums ship beside every artifact. `gpg` instructions will live next to the signing key on the release page.
  </p>
  <pre class="mono sh-block" aria-label="Checksum placeholder">sha256 stubhouse-macos-arm64.dmg
# compare to checksums.txt from the release</pre>
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
    padding: 16px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    font-size: 12px;
    line-height: 1.7;
    color: var(--text-secondary);
  }

  .pad-top {
    margin-top: 56px;
  }

  .sh-block {
    margin-top: 12px;
    padding: 16px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    font-size: 13px;
    line-height: 1.55;
    overflow: auto;
  }
</style>
