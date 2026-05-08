<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = { children: Snippet; class?: string };

  let { children, class: className = '' }: Props = $props();
</script>

<div class="section-reveal section-reveal--visible {className}">
  {@render children()}
</div>

<style>
  .section-reveal :global(.fade-up) {
    opacity: 0;
    transform: translateY(8px);
    transition:
      opacity 400ms cubic-bezier(0.22, 1, 0.36, 1),
      transform 400ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .section-reveal.section-reveal--visible :global(.fade-up) {
    opacity: 1;
    transform: translateY(0);
  }

  .section-reveal.section-reveal--visible :global(.fade-up.stagger-1) {
    transition-delay: 60ms;
  }

  .section-reveal.section-reveal--visible :global(.fade-up.stagger-2) {
    transition-delay: 120ms;
  }

  .section-reveal.section-reveal--visible :global(.fade-up.stagger-3) {
    transition-delay: 180ms;
  }

  @media (prefers-reduced-motion: reduce) {
    .section-reveal :global(.fade-up.stagger-1),
    .section-reveal :global(.fade-up.stagger-2),
    .section-reveal :global(.fade-up.stagger-3) {
      transition-delay: 0ms !important;
    }
  }
</style>
