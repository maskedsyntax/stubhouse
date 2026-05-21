<script lang="ts">
  import ArrowUp from 'lucide-svelte/icons/arrow-up';
  import { onMount } from 'svelte';

  let visible = $state(false);

  function scrollTop() {
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    window.scrollTo({ top: 0, behavior: reduce ? 'auto' : 'smooth' });
  }

  onMount(() => {
    const update = () => {
      visible = window.scrollY > Math.max(480, window.innerHeight * 0.7);
    };
    update();
    window.addEventListener('scroll', update, { passive: true });
    window.addEventListener('resize', update);
    return () => {
      window.removeEventListener('scroll', update);
      window.removeEventListener('resize', update);
    };
  });
</script>

<button
  type="button"
  class="back-top"
  class:back-top--visible={visible}
  aria-label="Back to top"
  onclick={scrollTop}
>
  <ArrowUp size={18} strokeWidth={1.8} aria-hidden="true" />
</button>

<style>
  .back-top {
    position: fixed;
    right: clamp(16px, 3vw, 32px);
    bottom: clamp(16px, 3vw, 32px);
    z-index: 90;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--bg-surface) 86%, transparent);
    color: var(--text-primary);
    box-shadow:
      inset 0 1px 0 var(--border-subtle),
      0 18px 48px rgba(0, 0, 0, 0.22);
    backdrop-filter: blur(14px) saturate(130%);
    cursor: pointer;
    opacity: 0;
    pointer-events: none;
    transform: translateY(8px);
    transition:
      opacity 160ms ease,
      transform 160ms var(--ease-out),
      border-color 160ms ease,
      background-color 160ms ease;
  }

  .back-top--visible {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }

  .back-top:hover,
  .back-top:focus-visible {
    border-color: var(--border-strong);
    background: var(--bg-surface);
  }

  @media (max-width: 639px) {
    .back-top {
      width: 40px;
      height: 40px;
      right: 14px;
      bottom: 14px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .back-top {
      transition: opacity 100ms ease;
      transform: none;
    }
  }
</style>
