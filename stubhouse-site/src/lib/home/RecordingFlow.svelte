<script lang="ts">
  let online = $state(true);
</script>

<div class="rec">
  <div class="rec__toolbar mono">
    <span class="rec__label">Upstream</span>
    <button
      type="button"
      class="rec__toggle"
      onclick={() => (online = !online)}
      aria-pressed={online}
    >
      {online ? 'Recording on' : 'Replay local'}
    </button>
  </div>
  <svg class="rec__svg" viewBox="0 0 640 120" aria-hidden="true">
    <path
      class="rec__line"
      d="M40 60 H260"
      fill="none"
      stroke="currentColor"
      stroke-width="1.2"
    />
    <path
      class="rec__line"
      d="M380 60 H600"
      fill="none"
      stroke="currentColor"
      stroke-width="1.2"
    />
    <rect x="260" y="36" width="120" height="48" fill="none" stroke="currentColor" stroke-width="1.2" />
    <text x="292" y="66" font-size="13" font-family="ui-monospace, SFMono-Regular, Menlo, monospace" fill="currentColor">StubHouse</text>
    {#if online}
      <text x="400" y="32" font-size="11" class="rec__cap" fill="currentColor" opacity="0.6">live API</text>
    {:else}
      <text x="400" y="32" font-size="11" class="rec__cap" fill="currentColor" opacity="0.6">saved YAML</text>
    {/if}
    <circle cx="40" cy="60" r="4" fill="var(--bg-canvas)" stroke="currentColor" stroke-width="1.2" />
    <circle cx="600" cy="60" r="4" fill="var(--bg-canvas)" stroke="currentColor" stroke-width="1.2" />
  </svg>
  <p class="caption rec__note">
    {online
      ? 'Traffic passes through to the real service while rules are captured.'
      : 'The same routes are satisfied from disk — no network required.'}
  </p>
</div>

<style>
  .rec {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-surface);
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.18);
  }

  .rec__toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-subtle);
    gap: 12px;
  }

  .rec__label {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .rec__toggle {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-canvas);
    color: var(--text-primary);
    font-size: 12px;
    padding: 6px 10px;
    cursor: pointer;
    transition: border-color 200ms ease;
  }

  .rec__toggle:hover,
  .rec__toggle:focus-visible {
    border-color: var(--border-strong);
  }

  .rec__svg {
    width: 100%;
    height: auto;
    color: var(--text-secondary);
    display: block;
  }

  @media (max-width: 639px) {
    .rec {
      border-radius: var(--radius-sm);
      box-shadow: 0 14px 36px rgba(0, 0, 0, 0.2);
    }

    .rec__toolbar {
      padding: 9px 10px;
    }

    .rec__toggle {
      padding: 6px 8px;
      font-size: 11px;
    }

    .rec__svg {
      width: 130%;
      margin-left: -15%;
      max-width: none;
    }

    .rec__note {
      padding: 10px;
    }
  }

  .rec__note {
    margin: 0;
    padding: 12px;
    border-top: 1px solid var(--border-subtle);
  }
</style>
