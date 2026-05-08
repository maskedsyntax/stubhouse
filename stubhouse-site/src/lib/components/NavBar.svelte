<script lang="ts">
  import { onMount } from 'svelte';
  import Button from './Button.svelte';
  import ThemeToggle from './ThemeToggle.svelte';

  type Props = { githubStars: number | null };
  let { githubStars }: Props = $props();

  let open = $state(false);
  let scrolled = $state(false);

  function closeNav() {
    open = false;
  }

  onMount(() => {
    const onScroll = () => {
      scrolled = window.scrollY > 8;
    };
    onScroll();
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') closeNav();
  }}
/>

<header class="nav" class:nav--scrolled={scrolled}>
  <div class="nav__inner container">
    <a class="nav__brand" href="/" onclick={closeNav} aria-label="StubHouse home">
      <img class="nav__mark" src="/stubhouse-logo-rounded-192.png" width="28" height="28" alt="" />
      <span class="nav__word">StubHouse</span>
    </a>

    <nav class="nav__links" aria-label="Primary">
      <a href="/mocks" onclick={closeNav}>Mocks</a>
      <a href="/pricing" onclick={closeNav}>Pricing</a>
      <a href="/docs" onclick={closeNav}>Docs</a>
      <a href="/changelog" onclick={closeNav}>Changelog</a>
      <a href="/blog" onclick={closeNav}>Blog</a>
    </nav>

    <div class="nav__actions">
      <a
        class="nav__gh link-inline mono nav__gh-link"
        href="https://github.com/stubhouse/stubhouse"
        target="_blank"
        rel="noopener noreferrer"
      >
        GitHub{#if githubStars != null}<span class="nav__stars">{githubStars.toLocaleString()}</span>{/if}
      </a>
      <ThemeToggle />
      <Button href="/download" variant="primary" size="md" class="nav__dl">Download</Button>
      <button
        type="button"
        class="nav__burger mono"
        aria-expanded={open}
        aria-controls="site-menu"
        onclick={() => (open = !open)}
      >
        Menu
      </button>
    </div>
  </div>

  <div
    id="site-menu"
    class="nav__drawer"
    class:nav__drawer--open={open}
    aria-hidden={!open}
  >
    <div class="container nav__drawer-inner">
      <a href="/mocks" onclick={closeNav}>Mocks</a>
      <a href="/pricing" onclick={closeNav}>Pricing</a>
      <a href="/docs" onclick={closeNav}>Docs</a>
      <a href="/changelog" onclick={closeNav}>Changelog</a>
      <a href="/blog" onclick={closeNav}>Blog</a>
      <a class="link-inline mono" href="https://github.com/stubhouse/stubhouse" target="_blank" rel="noopener noreferrer"
        >GitHub</a
      >
    </div>
  </div>
</header>

<style>
  .nav {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
    height: var(--nav-h);
    background: color-mix(in srgb, var(--bg-canvas) 72%, transparent);
    backdrop-filter: blur(18px) saturate(130%);
    border-bottom: 1px solid transparent;
    transition:
      border-color 150ms ease,
      background-color 150ms ease;
  }

  .nav--scrolled {
    border-bottom-color: var(--border-subtle);
    background: color-mix(in srgb, var(--bg-canvas) 88%, transparent);
  }

  .nav__inner {
    display: flex;
    align-items: center;
    height: var(--nav-h);
    gap: 20px;
  }

  .nav__brand {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    color: var(--text-primary);
    font-weight: 500;
    letter-spacing: -0.02em;
  }

  .nav__mark {
    display: block;
    border-radius: 7px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
  }

  .nav__links {
    display: none;
    align-items: center;
    gap: 6px;
    margin-left: 32px;
    flex: 1;
  }

  @media (min-width: 768px) {
    .nav__links {
      display: flex;
    }
  }

  .nav__links a {
    border-radius: var(--radius-sm);
    font-size: var(--text-body-sm);
    color: var(--text-secondary);
    text-decoration: none;
    padding: 7px 10px;
    transition: color 150ms ease;
  }

  .nav__links a:hover,
  .nav__links a:focus-visible {
    color: var(--text-primary);
    background: var(--shimmer);
  }

  .nav__actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .nav__gh-link {
    display: none;
    font-size: var(--text-body-sm);
    align-items: center;
    gap: 6px;
  }

  @media (min-width: 768px) {
    .nav__gh-link {
      display: inline-flex;
    }
  }

  .nav__stars {
    padding: 2px 8px;
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    font-size: 12px;
    color: var(--text-tertiary);
  }

  :global(.nav__dl.ui-btn) {
    display: none;
  }

  @media (min-width: 768px) {
    :global(.nav__dl.ui-btn) {
      display: inline-flex;
    }
  }

  .nav__burger {
    display: inline-flex;
    border: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    padding: 8px 10px;
    cursor: pointer;
    font-size: 12px;
    letter-spacing: 0.06em;
    font-variant-caps: all-small-caps;
  }

  @media (min-width: 768px) {
    .nav__burger {
      display: none;
    }
  }

  .nav__drawer {
    position: absolute;
    left: 0;
    right: 0;
    top: var(--nav-h);
    background: var(--bg-canvas);
    border-bottom: 1px solid var(--border-subtle);
    padding-block: 12px;
    transform: translateY(-12px);
    opacity: 0;
    pointer-events: none;
    transition:
      opacity 200ms ease,
      transform 200ms ease;
  }

  .nav__drawer--open {
    transform: translateY(0);
    opacity: 1;
    pointer-events: auto;
  }

  .nav__drawer-inner {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav__drawer-inner a {
    display: block;
    padding: 12px 0;
    color: var(--text-primary);
    text-decoration: none;
    font-size: var(--text-body);
  }

  @media (max-width: 639px) {
    .nav__inner {
      gap: 10px;
      padding-inline: 16px;
    }

    .nav__brand {
      gap: 8px;
      min-width: 0;
    }

    .nav__mark {
      width: 24px;
      height: 24px;
      border-radius: 6px;
    }

    .nav__word {
      font-size: 14px;
      max-width: 120px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .nav__actions {
      gap: 2px;
    }

    .nav__burger {
      min-height: 34px;
      padding: 0 9px;
      border-radius: 7px;
      font-family: var(--font-sans);
      font-size: 11px;
      letter-spacing: 0;
      font-variant-caps: normal;
    }

    .nav__drawer {
      background: color-mix(in srgb, var(--bg-canvas) 96%, transparent);
      backdrop-filter: blur(16px);
      padding-block: 8px;
    }

    .nav__drawer-inner a {
      padding: 14px 0;
      border-bottom: 1px solid var(--border-subtle);
    }
  }

  @media (min-width: 768px) {
    .nav__drawer {
      display: none;
    }
  }
</style>
