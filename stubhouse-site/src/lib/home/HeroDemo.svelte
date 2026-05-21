<script lang="ts">
  import Activity from 'lucide-svelte/icons/activity';
  import FileCode from 'lucide-svelte/icons/file-code';
  import Play from 'lucide-svelte/icons/play';
  import Server from 'lucide-svelte/icons/server';
  import DemoFrame from '$lib/components/DemoFrame.svelte';
  import { onMount } from 'svelte';

  let slide = $state(0);

  const scenarios = ['happy_path', 'empty', 'outage', 'slow'];
  const logs = [
    ['GET', '/users/u_42', '200', 'get-user'],
    ['GET', '/users', '200', 'list-users'],
    ['POST', '/orders', '201', 'create-order']
  ];

  const responseBySlide = [
    `// workspace ready
base_url = http://127.0.0.1:4000
env      = mock`,
    `200 OK · 128 ms
{
  "id": "u_42",
  "name": "Ada Lovelace",
  "plan": "team",
  "flags": ["beta", "billing"]
}`,
    `503 Service Unavailable · 18 ms
{
  "error": "maintenance",
  "retry_after": 45
}`,
    `200 OK · 920 ms
{
  "items": [],
  "scenario": "empty"
}`
  ];

  const activeScenario = $derived(scenarios[slide] ?? scenarios[0]);

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
      <div class="hero-ui__brand-row">
        <span class="hero-ui__status-dot hero-ui__status-dot--on"></span>
        <span>acme-api</span>
      </div>
      <div class="hero-ui__group">Collections</div>
      <div class="hero-ui__item hero-ui__item--hi">GET users/:id</div>
      <div class="hero-ui__item">GET users</div>
      <div class="hero-ui__item">POST orders</div>
      <div class="hero-ui__group">Mocks</div>
      <div class="hero-ui__item" class:hero-ui__item--hi={slide >= 1}>rules/users.yaml</div>
      <div class="hero-ui__item" class:hero-ui__item--hi={slide >= 2}>scenarios/outage</div>
      <div class="hero-ui__server">
        <span class="hero-ui__server-top">
          <Server size={14} strokeWidth={1.7} aria-hidden="true" />
          mock server
        </span>
        <strong>127.0.0.1:4000</strong>
        <span>6 rules · hot reload</span>
      </div>
    </aside>

    <section class="hero-ui__main">
      <div class="hero-ui__toolbar mono" aria-hidden="true">
        <div class="hero-ui__request">
          <span class="hero-ui__method">GET</span>
          <span class="hero-ui__url">/users/:id</span>
          <span class="hero-ui__env">mock</span>
        </div>
        <span class="hero-ui__run">
          <Play size={13} strokeWidth={1.9} aria-hidden="true" />
          send
        </span>
      </div>

      <div class="hero-ui__work">
        <div class="hero-ui__mock-panel">
          <div class="hero-ui__panel-head">
            <span>
              <FileCode size={14} strokeWidth={1.7} aria-hidden="true" />
              Mock rule
            </span>
            <span class="mono">priority 30</span>
          </div>
          <pre class="hero-ui__rule mono">{`matcher:
  method: GET
  path: /users/:id
response:
  status: 200
  body:
    id: "{{params.id}}"
scenarios:
  outage: 503
  slow: delay 900ms`}</pre>
          <div class="hero-ui__scenarios" role="presentation">
            {#each scenarios as scenario}
              <span class:hero-ui__scenario-chip--on={scenario === activeScenario}>{scenario}</span>
            {/each}
          </div>
        </div>

        <div class="hero-ui__response-panel">
          <div class="hero-ui__panel-head">
            <span>
              <Activity size={14} strokeWidth={1.7} aria-hidden="true" />
              Response
            </span>
            <span class="mono">{slide === 2 ? 'matched outage' : slide === 3 ? 'matched empty' : 'matched happy_path'}</span>
          </div>
          <pre class="hero-ui__resp mono">{responseBySlide[slide]}</pre>
          <div class="hero-ui__assertions mono">
            <span class="hero-ui__check">status {slide === 2 ? '503' : '200'}</span>
            <span>route exact &gt; :param &gt; *</span>
            <span>JUnit ready</span>
          </div>
        </div>
      </div>

      <div class="hero-ui__bottom mono" aria-hidden="true">
        <div>
          <span class="hero-ui__bottom-title">Live mock log</span>
          <div class="hero-ui__log">
            {#each logs as log, index}
              <span class:hero-ui__log-row--active={index === slide % logs.length}>
                <b>{log[0]}</b> {log[1]} <em>{log[2]}</em> <small>{log[3]}</small>
              </span>
            {/each}
          </div>
        </div>
        <div class="hero-ui__metrics">
          <span><strong>6</strong> rules</span>
          <span><strong>{slide + 1}</strong> scenario</span>
          <span><strong>0</strong> cloud calls</span>
        </div>
      </div>
    </section>
  </div>
</DemoFrame>

<style>
  .hero-ui {
    display: grid;
    min-height: 360px;
  }

  @media (max-width: 639px) {
    .hero-ui {
      min-height: auto;
      width: 100%;
      min-width: 0;
    }
  }

  @media (min-width: 768px) {
    .hero-ui {
      grid-template-columns: 214px 1fr;
      min-height: 492px;
    }
  }

  .hero-ui__side {
    border-bottom: 1px solid var(--border-subtle);
    padding: 14px;
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

  .hero-ui__brand-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 30px;
    padding: 0 8px 8px;
    color: var(--text-primary);
  }

  .hero-ui__status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-disabled);
  }

  .hero-ui__status-dot--on {
    background: var(--text-primary);
    box-shadow: 0 0 0 4px var(--shimmer);
  }

  .hero-ui__group {
    margin: 12px 0 7px;
    font-variant-caps: all-small-caps;
    letter-spacing: 0.08em;
  }

  .hero-ui__item {
    padding: 7px 9px;
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

  .hero-ui__server {
    margin-top: 16px;
    display: grid;
    gap: 5px;
    padding: 11px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-secondary);
  }

  .hero-ui__server-top {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--text-primary);
  }

  .hero-ui__server strong {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .hero-ui__main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .hero-ui__toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 12px;
  }

  .hero-ui__request {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .hero-ui__method,
  .hero-ui__env,
  .hero-ui__run {
    border: 1px solid var(--border-default);
    border-radius: 999px;
    background: var(--bg-surface);
  }

  .hero-ui__method {
    padding: 2px 8px;
    color: var(--text-primary);
  }

  .hero-ui__env {
    padding: 2px 7px;
    color: var(--text-tertiary);
  }

  .hero-ui__url {
    color: var(--text-secondary);
    min-width: 0;
  }

  .hero-ui__run {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 9px;
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .hero-ui__work {
    flex: 1;
    display: grid;
    gap: 12px;
    padding: clamp(12px, 2vw, 18px);
    min-height: 0;
  }

  @media (min-width: 980px) {
    .hero-ui__work {
      grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
    }
  }

  .hero-ui__mock-panel,
  .hero-ui__response-panel {
    min-width: 0;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background:
      linear-gradient(to bottom, color-mix(in srgb, var(--bg-surface-2) 34%, transparent), transparent),
      var(--bg-surface);
    overflow: hidden;
  }

  .hero-ui__panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-size: 12px;
  }

  .hero-ui__panel-head span:first-child {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-primary);
  }

  .hero-ui__rule,
  .hero-ui__resp {
    margin: 0;
    padding: 14px;
    font-size: 12px;
    line-height: 1.55;
    white-space: pre-wrap;
    overflow: auto;
  }

  .hero-ui__resp {
    min-height: 184px;
    color: var(--text-primary);
  }

  .hero-ui__rule {
    color: var(--text-secondary);
  }

  .hero-ui__scenarios,
  .hero-ui__assertions {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    padding: 0 14px 14px;
  }

  .hero-ui__scenarios span,
  .hero-ui__assertions span {
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 4px 8px;
    color: var(--text-tertiary);
    font-size: 11px;
    line-height: 1.2;
  }

  .hero-ui__scenarios .hero-ui__scenario-chip--on,
  .hero-ui__check {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--shimmer);
  }

  .hero-ui__bottom {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 12px;
    align-items: end;
    padding: 0 clamp(12px, 2vw, 18px) clamp(12px, 2vw, 18px);
    font-size: 11px;
  }

  .hero-ui__bottom-title {
    display: block;
    margin-bottom: 7px;
    color: var(--text-tertiary);
  }

  .hero-ui__log {
    display: grid;
    gap: 4px;
  }

  .hero-ui__log span {
    display: grid;
    grid-template-columns: 40px minmax(0, 1fr) 36px 80px;
    gap: 8px;
    align-items: center;
    min-height: 24px;
    padding: 3px 8px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .hero-ui__log-row--active {
    border-color: var(--border-subtle) !important;
    background: var(--shimmer);
    color: var(--text-primary) !important;
  }

  .hero-ui__log b,
  .hero-ui__log em,
  .hero-ui__metrics strong {
    color: var(--text-primary);
    font-style: normal;
    font-weight: 500;
  }

  .hero-ui__log small {
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hero-ui__metrics {
    display: grid;
    grid-template-columns: repeat(3, auto);
    gap: 8px;
  }

  .hero-ui__metrics span {
    display: grid;
    place-items: center;
    min-width: 64px;
    min-height: 48px;
    padding: 7px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
  }

  @media (max-width: 639px) {
    .hero-ui__toolbar {
      padding: 9px 10px;
      font-size: 11px;
    }

    .hero-ui__url {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .hero-ui__env,
    .hero-ui__run,
    .hero-ui__mock-panel,
    .hero-ui__bottom {
      display: none;
    }

    .hero-ui__work {
      padding: 10px;
      display: block;
    }

    .hero-ui__panel-head {
      padding: 9px 10px;
      font-size: 11px;
    }

    .hero-ui__resp {
      min-height: 178px;
      padding: 10px;
      font-size: 11px;
      line-height: 1.55;
    }

    .hero-ui__assertions {
      padding: 0 10px 10px;
      gap: 5px;
    }

    .hero-ui__assertions span {
      font-size: 10px;
      padding: 3px 6px;
    }
  }
</style>
