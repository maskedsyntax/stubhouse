<script lang="ts">
  import Moon from 'lucide-svelte/icons/moon';
  import Sun from 'lucide-svelte/icons/sun';
  import { onMount } from 'svelte';

  type Pref = 'dark' | 'light' | 'system';

  function readStored(): Pref {
    if (typeof localStorage === 'undefined') return 'system';
    const stored = localStorage.getItem('stubhouse-theme') as Pref | null;
    if (stored === 'dark' || stored === 'light' || stored === 'system') return stored;
    return 'system';
  }

  function resolvedClass(prefVal: Pref): string {
    if (prefVal === 'system') {
      return window.matchMedia('(prefers-color-scheme: light)').matches ? 'theme-light' : 'theme-dark';
    }
    return prefVal === 'light' ? 'theme-light' : 'theme-dark';
  }

  function apply(prefVal: Pref) {
    document.documentElement.className = resolvedClass(prefVal);
    document.documentElement.setAttribute('data-theme-pref', prefVal);
    localStorage.setItem('stubhouse-theme', prefVal);
    pref = prefVal;
  }

  let pref = $state<Pref>('system');
  let showSun = $state(true);

  function syncResolvedIcon() {
    showSun = document.documentElement.classList.contains('theme-dark');
  }

  onMount(() => {
    pref = readStored();
    syncResolvedIcon();
    const mq = window.matchMedia('(prefers-color-scheme: light)');
    const onOs = () => {
      if (readStored() === 'system') {
        document.documentElement.className = resolvedClass('system');
        syncResolvedIcon();
      }
    };
    mq.addEventListener('change', onOs);
    const mo = new MutationObserver(syncResolvedIcon);
    mo.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => {
      mq.removeEventListener('change', onOs);
      mo.disconnect();
    };
  });

  function cycle() {
    const order: Pref[] = ['dark', 'light', 'system'];
    const next = order[(order.indexOf(pref) + 1) % order.length];
    apply(next);
  }

  const label = $derived(
    pref === 'dark'
      ? 'Color theme preference: dark. Cycles to light.'
      : pref === 'light'
        ? 'Color theme preference: light. Cycles to system.'
        : 'Color theme preference: system. Cycles to dark.'
  );
</script>

<button type="button" class="theme-toggle" onclick={cycle} aria-label={label}>
  {#if showSun}
    <Sun size={18} strokeWidth={1.75} aria-hidden="true" />
  {:else}
    <Moon size={18} strokeWidth={1.75} aria-hidden="true" />
  {/if}
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border: 0;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 0;
    transition: color 150ms ease;
  }

  .theme-toggle:hover,
  .theme-toggle:focus-visible {
    color: var(--text-primary);
  }

  @media (max-width: 639px) {
    .theme-toggle {
      width: 34px;
      height: 34px;
    }
  }
</style>
