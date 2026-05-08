<script lang="ts">
  import DemoFrame from '$lib/components/DemoFrame.svelte';
  import { onMount } from 'svelte';

  let slide = $state(0);

  onMount(() => {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    let s = 0;
    const id = window.setInterval(() => {
      s = (s + 1) % 4;
      slide = s;
    }, 3200);
    return () => window.clearInterval(id);
  });
</script>

<DemoFrame title="StubHouse — my-project">
  <div class="hero-ui">
    <aside class="hero-ui__side mono" aria-hidden="true">
      <div class="hero-ui__group">Users</div>
      <div class="hero-ui__item" class:hero-ui__item--hi={slide >= 1}>GET list</div>
      <div class="hero-ui__item">POST create</div>
      <div class="hero-ui__group">Mock</div>
      <div class="hero-ui__item" class:hero-ui__item--hi={slide >= 2}>GET /users/:id</div>
      <div class="hero-ui__pill" class:hero-ui__pill--on={slide >= 2}>
        mock server <span class="hero-ui__dot" class:hero-ui__dot--on={slide >= 2}></span>
      </div>
      <div class="hero-ui__meta mono">127.0.0.1:4000</div>
    </aside>
    <section class="hero-ui__main">
      <div class="hero-ui__bar mono">
        <span class="hero-ui__method">GET</span>
        <span class="hero-ui__url">https://api.example.com/users</span>
      </div>
      <div class="hero-ui__body">
        <p class="caption">Response</p>
        <div class="hero-ui__resp mono">
          {#if slide === 0}
            <pre>{`// Open workspace`}</pre>
          {:else if slide === 1}
            <pre>{`200 OK · 128 ms\n{\n  "items": [\n    { "id": "u1", "name": "Ada" }\n  ]\n}`}</pre>
          {:else if slide === 2}
            <pre>{`mock active → localhost`}</pre>
          {:else}
            <pre>{`404 Not Found\n{\n  "error": "User not found"\n}`}</pre>
          {/if}
        </div>
        {#if slide >= 3}
          <p class="hero-ui__scenario caption">Scenario: <span class="mono">not_found</span></p>
        {:else if slide >= 2}
          <p class="hero-ui__scenario caption">Scenario: <span class="mono">success</span></p>
        {/if}
      </div>
    </section>
  </div>
</DemoFrame>

<style>
  .hero-ui {
    display: grid;
    min-height: 280px;
  }

  @media (max-width: 639px) {
    .hero-ui {
      min-height: auto;
    }
  }

  @media (min-width: 768px) {
    .hero-ui {
      grid-template-columns: 220px 1fr;
      min-height: 440px;
    }
  }

  .hero-ui__side {
    border-bottom: 1px solid var(--border-subtle);
    padding: 16px;
    font-size: 11px;
    color: var(--text-tertiary);
    background: color-mix(in srgb, var(--bg-surface) 58%, transparent);
  }

  @media (max-width: 639px) {
    .hero-ui__side {
      display: none;
    }
  }

  @media (min-width: 768px) {
    .hero-ui__side {
      border-bottom: none;
      border-right: 1px solid var(--border-subtle);
    }
  }

  .hero-ui__group {
    margin: 10px 0 7px;
    font-variant-caps: all-small-caps;
    letter-spacing: 0.08em;
  }

  .hero-ui__item {
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    border-left: 2px solid transparent;
    transition:
      background-color 220ms ease,
      color 220ms ease,
      border-color 220ms ease;
  }

  .hero-ui__item--hi {
    color: var(--text-primary);
    border-left-color: var(--text-primary);
    background: var(--shimmer);
  }

  .hero-ui__pill {
    margin-top: 14px;
    padding: 10px;
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--text-tertiary);
  }

  .hero-ui__pill--on {
    border-style: solid;
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .hero-ui__dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1px solid var(--border-default);
  }

  .hero-ui__dot--on {
    background: var(--text-primary);
  }

  .hero-ui__meta {
    margin-top: 8px;
    font-size: 10px;
    color: var(--text-disabled);
  }

  .hero-ui__main {
    display: flex;
    flex-direction: column;
  }

  .hero-ui__bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 12px;
  }

  @media (max-width: 639px) {
    .hero-ui__bar {
      flex-wrap: nowrap;
      padding: 10px;
      gap: 6px;
      font-size: 11px;
    }

    .hero-ui__method {
      flex: 0 0 auto;
    }

    .hero-ui__url {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }

  .hero-ui__method {
    padding: 2px 8px;
    border: 1px solid var(--border-default);
    border-radius: 999px;
  }

  .hero-ui__url {
    color: var(--text-secondary);
  }

  .hero-ui__body {
    padding: clamp(14px, 2vw, 24px);
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  @media (max-width: 639px) {
    .hero-ui__body {
      padding: 12px;
    }
  }

  .hero-ui__resp {
    flex: 1;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: clamp(14px, 2vw, 22px);
    background:
      linear-gradient(to bottom, color-mix(in srgb, var(--bg-surface-2) 38%, transparent), transparent),
      var(--bg-surface);
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
    overflow: auto;
  }

  @media (max-width: 639px) {
    .hero-ui__resp {
      min-height: 156px;
      max-height: 220px;
      padding: 12px;
      font-size: 11px;
      line-height: 1.55;
    }
  }

  .hero-ui__resp pre {
    margin: 0;
    white-space: pre-wrap;
  }

  .hero-ui__scenario {
    margin: 0;
  }
</style>
