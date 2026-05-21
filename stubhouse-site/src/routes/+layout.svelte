<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import NavBar from '$lib/components/NavBar.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import BackToTop from '$lib/components/BackToTop.svelte';

  let { data, children } = $props();

  onMount(() => {
    const bar = document.querySelector('.scroll-progress') as HTMLElement | null;
    if (!bar) return;
    const update = () => {
      const h = document.documentElement;
      const max = h.scrollHeight - h.clientHeight;
      const p = max <= 0 ? 0 : (h.scrollTop / max) * 100;
      if (bar) bar.style.width = `${p}%`;
    };
    update();
    window.addEventListener('scroll', update, { passive: true });
    return () => window.removeEventListener('scroll', update);
  });
</script>

<svelte:head>
  <meta name="theme-color" content="#0a0a0a" />
</svelte:head>

<div class="scroll-progress" aria-hidden="true"></div>
<a class="skip-link" href="#main">Skip to content</a>
<NavBar githubStars={data.githubStars} />
<main id="main" class="main">
  {@render children()}
</main>
<Footer />
<BackToTop />

<style>
  .main {
    padding-top: var(--nav-h);
  }
</style>
