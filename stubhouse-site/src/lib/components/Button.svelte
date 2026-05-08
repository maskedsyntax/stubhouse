<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'secondary' | 'ghost';
  type Size = 'sm' | 'md' | 'lg';

  type Props = {
    href?: string;
    variant?: Variant;
    size?: Size;
    type?: 'button' | 'submit';
    class?: string;
    children: Snippet;
    lead?: Snippet;
    trailing?: Snippet;
    ['aria-label']?: string;
    target?: string;
    rel?: string;
  };

  let {
    href,
    variant = 'primary',
    size = 'md',
    type = 'button',
    class: className = '',
    children,
    lead,
    trailing,
    'aria-label': ariaLabel,
    target,
    rel
  }: Props = $props();

  const cls = $derived(`ui-btn ui-btn--${variant} ui-btn--${size} ${className}`.trim());
</script>

{#if href}
  <a
    class={cls}
    {href}
    {target}
    rel={rel ?? (target === '_blank' ? 'noopener noreferrer' : undefined)}
    aria-label={ariaLabel}
  >
    {#if lead}{@render lead()}{/if}
    {@render children()}
    {#if trailing}{@render trailing()}{/if}
  </a>
{:else}
  <button class={cls} {type} aria-label={ariaLabel}>
    {#if lead}{@render lead()}{/if}
    {@render children()}
    {#if trailing}{@render trailing()}{/if}
  </button>
{/if}

<style>
  .ui-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-weight: 500;
    font-size: var(--text-body-sm);
    letter-spacing: 0;
    text-decoration: none;
    cursor: pointer;
    border: 1px solid transparent;
    transition:
      background-color 200ms ease,
      border-color 200ms ease,
      color 200ms ease,
      transform 200ms var(--ease-out),
      box-shadow 200ms ease;
    white-space: nowrap;
  }

  .ui-btn:hover {
    transform: translateY(-1px);
  }

  .ui-btn:focus-visible {
    outline: 2px solid var(--text-primary);
    outline-offset: 2px;
  }

  .ui-btn--sm {
    min-height: 32px;
    padding: 0 12px;
  }
  .ui-btn--md {
    min-height: 40px;
    padding: 0 16px;
  }
  .ui-btn--lg {
    min-height: 48px;
    padding: 0 20px;
    font-size: var(--text-body);
  }

  .ui-btn--primary {
    background: var(--text-primary);
    color: var(--bg-canvas);
    border-color: transparent;
    box-shadow:
      inset 0 -1px 0 color-mix(in srgb, var(--bg-canvas) 24%, transparent),
      0 12px 32px rgba(0, 0, 0, 0.18);
  }

  .ui-btn--primary:hover {
    filter: brightness(0.92);
  }

  .ui-btn--secondary {
    background: color-mix(in srgb, var(--bg-surface) 74%, transparent);
    color: var(--text-primary);
    border-color: var(--border-default);
    box-shadow: inset 0 1px 0 var(--border-subtle);
  }

  .ui-btn--secondary:hover {
    border-color: var(--border-strong);
  }

  .ui-btn--ghost {
    background: transparent;
    color: var(--text-secondary);
    border-color: transparent;
  }

  .ui-btn--ghost:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  :global(.ui-btn svg) {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  @media (max-width: 639px) {
    .ui-btn {
      width: 100%;
      min-width: 0;
      white-space: normal;
      text-align: center;
    }

    .ui-btn--lg {
      min-height: 46px;
      padding-inline: 16px;
      font-size: var(--text-body-sm);
    }
  }
</style>
