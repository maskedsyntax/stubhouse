<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title = 'StubHouse',
    class: className = '',
    children,
  }: {
    title?: string;
    class?: string;
    children: Snippet;
  } = $props();
</script>

<div
  class="demo-frame {className}"
  role="region"
  aria-label={title}
>
  <div class="demo-chrome">
    <span class="demo-lights" aria-hidden="true">
      <span></span>
      <span></span>
      <span></span>
    </span>
    <span class="demo-title mono">{title}</span>
    <span class="demo-spacer" aria-hidden="true"></span>
  </div>
  <div class="demo-body">
    {@render children()}
  </div>
</div>

<style>
  .demo-frame {
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
    overflow: hidden;
    background: var(--bg-surface-2);
    box-shadow:
      0 0 0 1px var(--border-subtle),
      var(--shadow-soft);
  }

  .demo-chrome {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
    background:
      linear-gradient(to bottom, color-mix(in srgb, var(--bg-surface-2) 64%, transparent), transparent),
      var(--bg-surface);
  }

  .demo-lights {
    display: flex;
    gap: 6px;
  }

  .demo-lights span {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 1px solid var(--border-strong);
    background: var(--bg-surface-2);
  }

  .demo-lights span:nth-child(1) {
    opacity: 0.7;
  }

  .demo-lights span:nth-child(2) {
    opacity: 0.45;
  }

  .demo-lights span:nth-child(3) {
    opacity: 0.25;
  }

  .demo-title {
    flex: 1;
    text-align: center;
    font-size: var(--text-caption);
    color: var(--text-tertiary);
  }

  .demo-spacer {
    width: 52px;
  }

  .demo-body {
    background: var(--bg-canvas);
  }

  @media (max-width: 639px) {
    .demo-frame {
      border-radius: var(--radius-sm);
      box-shadow:
        0 0 0 1px var(--border-subtle),
        0 16px 42px rgba(0, 0, 0, 0.22);
    }

    .demo-chrome {
      padding: 0.5rem 0.6rem;
    }

    .demo-lights {
      display: none;
    }

    .demo-title {
      text-align: left;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .demo-spacer {
      display: none;
    }
  }
</style>
