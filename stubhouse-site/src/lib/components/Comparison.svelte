<script lang="ts">
  import comparison from '$lib/data/comparison.json';

  type Cell = boolean | string;

  function cellText(v: Cell): string {
    if (v === true) return '✓';
    if (v === false) return '✕';
    if (v === 'partial') return '◐';
    return String(v);
  }

  let { class: className = '' } = $props();
</script>

<div class="comparison-wrap {className}">
  <div class="comparison-scroll" role="region" aria-label="Product comparison">
    <table class="comparison">
      <thead>
        <tr>
          <th scope="col" class="comparison__feat"></th>
          <th scope="col" class="comparison__sh">StubHouse</th>
          <th scope="col">Postman</th>
          <th scope="col">Insomnia</th>
          <th scope="col">Bruno</th>
          <th scope="col">Yaak</th>
        </tr>
      </thead>
      <tbody>
        {#each comparison.rows as row}
          <tr>
            <th scope="row" class="comparison__feat">{row.feature}</th>
            <td class="comparison__sh mono">{cellText(row.stubhouse as Cell)}</td>
            <td class="mono">{cellText(row.postman as Cell)}</td>
            <td class="mono">{cellText(row.insomnia as Cell)}</td>
            <td class="mono">{cellText(row.bruno as Cell)}</td>
            <td class="mono">{cellText(row.yaak as Cell)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <div class="comparison-mobile" aria-label="Product comparison">
    {#each comparison.rows as row}
      <article class="comparison-card">
        <h3>{row.feature}</h3>
        <div class="comparison-card__grid">
          <span>StubHouse</span>
          <strong>{cellText(row.stubhouse as Cell)}</strong>
          <span>Postman</span>
          <strong>{cellText(row.postman as Cell)}</strong>
          <span>Insomnia</span>
          <strong>{cellText(row.insomnia as Cell)}</strong>
          <span>Bruno</span>
          <strong>{cellText(row.bruno as Cell)}</strong>
          <span>Yaak</span>
          <strong>{cellText(row.yaak as Cell)}</strong>
        </div>
      </article>
    {/each}
  </div>
  <p class="caption comparison__note">
    Sourced from public information as of {comparison.updated}. Updated when competitors ship. Corrections welcome — open a PR.
  </p>
</div>

<style>
  .comparison-wrap {
    width: 100%;
    min-width: 0;
  }

  .comparison-scroll {
    overflow-x: auto;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    -webkit-overflow-scrolling: touch;
    box-shadow: 0 18px 56px rgba(0, 0, 0, 0.16);
  }

  .comparison {
    width: 100%;
    min-width: 720px;
    border-collapse: collapse;
    font-size: var(--text-body-sm);
  }

  .comparison th,
  .comparison td {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
    text-align: left;
    vertical-align: top;
    color: var(--text-secondary);
    font-weight: 400;
  }

  .comparison thead th {
    color: var(--text-primary);
    font-weight: 500;
    border-bottom-color: var(--border-default);
    background:
      linear-gradient(to bottom, color-mix(in srgb, var(--bg-surface) 28%, transparent), transparent),
      var(--bg-surface-2);
  }

  .comparison__feat {
    width: 28%;
    color: var(--text-primary);
    font-weight: 500;
  }

  .comparison__sh {
    background: var(--shimmer);
    color: var(--text-primary);
  }

  .comparison tbody th {
    font-weight: 400;
    color: var(--text-primary);
  }

  .comparison__note {
    margin-top: 16px;
    max-width: 65ch;
  }

  .comparison-mobile {
    display: none;
  }

  @media (max-width: 639px) {
    .comparison-scroll {
      display: none;
    }

    .comparison-mobile {
      display: grid;
      gap: 8px;
    }

    .comparison-card {
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-sm);
      background: var(--bg-surface);
      overflow: hidden;
    }

    .comparison-card h3 {
      margin: 0;
      padding: 10px 12px;
      border-bottom: 1px solid var(--border-subtle);
      color: var(--text-primary);
      background: var(--bg-surface-2);
      font-size: 13px;
      line-height: 1.35;
      font-weight: 500;
    }

    .comparison-card__grid {
      display: grid;
      grid-template-columns: 1fr auto;
      gap: 0;
      font-size: 12px;
    }

    .comparison-card__grid span,
    .comparison-card__grid strong {
      padding: 8px 12px;
      border-bottom: 1px solid var(--border-subtle);
    }

    .comparison-card__grid span {
      color: var(--text-secondary);
    }

    .comparison-card__grid strong {
      min-width: 64px;
      text-align: right;
      color: var(--text-primary);
      font-family: var(--font-mono);
      font-size: 12px;
      font-weight: 400;
    }

    .comparison-card__grid span:nth-last-child(-n + 2),
    .comparison-card__grid strong:nth-last-child(-n + 2) {
      border-bottom: 0;
    }

    .comparison__note {
      margin-top: 12px;
    }
  }
</style>
