<script lang="ts">
  import Button from '$lib/components/Button.svelte';
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import Comparison from '$lib/components/Comparison.svelte';
  import DemoFrame from '$lib/components/DemoFrame.svelte';
  import SectionReveal from '$lib/SectionReveal.svelte';
  import HeroDemo from '$lib/home/HeroDemo.svelte';
  import RecordingFlow from '$lib/home/RecordingFlow.svelte';
  import ScenarioDemo from '$lib/home/ScenarioDemo.svelte';

  const meta = {
    title: 'StubHouse — local-first API client and mock server',
    description:
      'StubHouse is the desktop API client that takes mocking seriously. Native, offline, file-based — for the developer who ships before the API exists.'
  };

  const yamlExample = `id: req_get_user
name: "Get User by ID"
description: |
  Retrieves a single user by their unique ID.
method: GET
url: "{{base_url}}/users/{{user_id}}"
headers:
  - key: Authorization
    value: "Bearer {{auth_token}}"
    enabled: true
auth:
  type: bearer
  token: "{{auth_token}}"
tags:
  - users
  - read`;

  const rhaiExample = `test("Status is 200")        { response.status == 200 }
test("Body has user field")  { response.json()["user"] != null }
test("Response under 500ms") { response.time_ms < 500 }`;

  const ciExample = `# CI: serve mocks, run tests, switch scenario
stubhouse serve . --port 4000 --env test &
MOCK_PID=$!

npm test

stubhouse scenario set mock_payment checkout_failure
npm test -- --grep "error handling"

kill $MOCK_PID`;

  const terminalBuild = `$ git clone https://github.com/stubhouse/stubhouse
$ cd stubhouse && cargo build --release
   Finished release [optimized] target(s) in 2m 14s`;

  const curlExample = `curl -X POST http://127.0.0.1:4000/users -d '{"name":"Ada"}'
curl http://127.0.0.1:4000/users/usr_1
curl -X DELETE http://127.0.0.1:4000/users/usr_1`;

  const faults = [
    {
      name: 'connection_reset',
      hint: 'TCP reset mid-response — exercise reconnect paths.',
      yaml: 'fault: connection_reset'
    },
    {
      name: 'timeout',
      hint: 'No reply until the client gives up.',
      yaml: 'fault: timeout'
    },
    {
      name: 'slow_response',
      hint: 'Fixed or random tail latency.',
      yaml: 'delay_ms:\n  type: random\n  min: 800\n  max: 3000'
    },
    {
      name: 'partial_body',
      hint: 'Truncated payloads for parser hardening.',
      yaml: 'fault: partial_body'
    },
    {
      name: 'random_5xx',
      hint: 'Rare server errors on demand.',
      yaml: 'fault:\n  type: random_5xx\n  probability: 0.12'
    },
    {
      name: 'passthrough',
      hint: 'Forward to upstream when you need a real edge.',
      yaml: 'passthrough: true'
    }
  ];
</script>

<svelte:head>
  <title>{meta.title}</title>
  <meta name="description" content={meta.description} />
  <meta property="og:title" content={meta.title} />
  <meta property="og:description" content={meta.description} />
  <meta property="og:type" content="website" />
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={meta.title} />
  <meta name="twitter:description" content={meta.description} />
</svelte:head>

<section class="hero">
  <div class="container hero__inner">
    <div class="hero__mark">
      <img src="/stubhouse-logo-rounded-512.png" width="88" height="88" alt="" />
    </div>
    <p class="eyebrow hero-ent hero-ent--1">Local-first API client + mock server</p>
    <h1 class="display-1 hero__title hero-ent hero-ent--2">Stub it. Ship it.</h1>
    <p class="body-lg hero__sub hero-ent hero-ent--3">
      StubHouse is the desktop API client that takes mocking seriously. Native, offline, file-based — built for the developer who
      has to ship before the API exists.
    </p>
    <div class="hero__cta hero-ent hero-ent--4">
      <Button href="/download" variant="primary" size="lg" class="hero__cta-btn">
        {#snippet lead()}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
            <path d="M12 4v12m0 0 4-4m-4 4-4-4M4 20h16" />
          </svg>
        {/snippet}
        Download for macOS
      </Button>
      <Button
        href="https://github.com/stubhouse/stubhouse"
        variant="secondary"
        size="lg"
        target="_blank"
        rel="noopener noreferrer"
      >
        View on GitHub
        {#snippet trailing()}
          <span aria-hidden="true">→</span>
        {/snippet}
      </Button>
    </div>
    <p class="caption hero__platforms hero-ent hero-ent--4">Available for macOS, Linux, and Windows.</p>
    <div class="hero__proof hero-ent hero-ent--5" aria-label="Product highlights">
      <span><strong>Native</strong> Rust core</span>
      <span><strong>Offline</strong> workspaces</span>
      <span><strong>Mock</strong> server included</span>
    </div>
    <div class="hero__demo hero-ent hero-ent--5">
      <HeroDemo />
      <p class="caption hero__demo-note">
        One workspace for real requests, local mock rules, scenarios, and recorded fixtures.
      </p>
    </div>
  </div>
</section>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <p class="eyebrow fade-up">Two tools. One binary.</p>
      <h2 class="display-2 fade-up stagger-1">A request client and a mock server, both first-class.</h2>
      <p class="body-lg fade-up stagger-2">
        Other API clients bolt on mocks as a paid afterthought or a separate process. StubHouse was built around the assumption
        that you spend half your day calling APIs and the other half pretending an API exists. Both deserve a real tool.
      </p>
    </div>
    <div class="dual fade-up stagger-2">
      <DemoFrame title="Request — production">
        <div class="dual__panel">
          <p class="mono dual__bar">GET · https://api.example.com/users</p>
          <pre class="mono dual__pre">{`200 OK · 132 ms
{
  "items": [{ "id": "u1", "name": "Ada" }]
}`}</pre>
        </div>
      </DemoFrame>
      <DemoFrame title="Mocks — rules">
        <div class="dual__panel">
          <ul class="mono dual__list">
            <li>GET /users <span class="dual__tag">success</span></li>
            <li>POST /users <span class="dual__tag">success</span></li>
            <li>GET /users/:id <span class="dual__tag">not_found</span></li>
          </ul>
          <p class="caption dual__hint">Scenario dropdowns live next to the routes they own.</p>
        </div>
      </DemoFrame>
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <p class="eyebrow fade-up">Source of truth</p>
      <h2 class="display-2 fade-up stagger-1">Your API definitions live in a folder you own.</h2>
      <p class="body-lg fade-up stagger-2">
        No proprietary database. No cloud account required. A workspace is a `.stubhouse/` directory of YAML files: requests,
        mock rules, environments, scripts. Commit it to git. Diff it in PRs. Move it between machines with `cp -r`. The tool builds
        on top of files; the files outlive the tool.
      </p>
    </div>
    <div class="fade-up stagger-2">
      <CodeBlock filename=".stubhouse/collections/users/get-user.yaml" language="yaml" code={yamlExample} />
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container mock-head">
    <p class="eyebrow fade-up">The mock server</p>
    <h2 class="display-2 fade-up stagger-1">Three things worth doing in-process.</h2>
  </div>
  <div class="container mock-rows">
    <article class="mock-row">
      <div>
        <h3 class="display-3 fade-up">Scenarios</h3>
        <p class="body-lg fade-up stagger-1">
          Switch your mock from `success` to `not_found` to `server_error` without restarting anything. From the UI, from the CLI,
          or from your test runner via the control API.
        </p>
      </div>
      <div class="fade-up stagger-2">
        <ScenarioDemo />
      </div>
    </article>
    <article class="mock-row mock-row--flip">
      <div>
        <h3 class="display-3 fade-up">Stateful mocks</h3>
        <p class="body-lg fade-up stagger-1">
          A mock that behaves like an API. POST creates. PUT updates. DELETE removes. GET returns what you wrote. All in memory, all
          reset on demand.
        </p>
      </div>
      <div class="fade-up stagger-2">
        <CodeBlock filename=".stubhouse/collections/users/mocks/resources.yaml" language="yaml" code={`mock_resources:\n  - path: /users\n    id_field: id\n    seed_file: ./fixtures/users.yaml\n    auto_crud: true`} />
        <pre class="mono curl-block" aria-label="Example curls">{curlExample}</pre>
      </div>
    </article>
    <article class="mock-row">
      <div>
        <h3 class="display-3 fade-up">Recording mode</h3>
        <p class="body-lg fade-up stagger-1">
          Point StubHouse at a real API. Make some calls. Save them. Replay them offline. Forever.
        </p>
      </div>
      <div class="fade-up stagger-2">
        <RecordingFlow />
      </div>
    </article>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container">
    <p class="eyebrow fade-up">Chaos, on demand</p>
    <h2 class="display-2 fade-up stagger-1">Test the unhappy path before production does.</h2>
    <p class="body-lg prose-width fade-up stagger-2">
      Connection resets. Timeouts. Slow responses. Partial bodies. Random 5xx at a configurable rate. Every fault is a checkbox. Your
      retry logic finally has something to retry against.
    </p>
    <div class="fault-grid fade-up stagger-3">
      {#each faults as f}
        <div class="fault-card" title={f.hint}>
          <p class="mono fault-card__name">{f.name}</p>
          <p class="caption fault-card__hint">{f.hint}</p>
          <pre class="mono fault-card__yaml">{f.yaml}</pre>
        </div>
      {/each}
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <p class="eyebrow fade-up">Automation, sandboxed</p>
      <h2 class="display-2 fade-up stagger-1">Logic that doesn't need a Node.js runtime.</h2>
      <p class="body-lg fade-up stagger-2">
        StubHouse uses Rhai — a sandboxed scripting language designed for embedding in Rust. Pre-request scripts. Post-response
        assertions. Mock rule conditions. All run in a sandbox with no filesystem, no network, no escape. No 50MB V8 binary in your
        dock.
      </p>
    </div>
    <div class="automation-stack fade-up stagger-2">
      <CodeBlock filename="tests.rhai" language="rhai" code={rhaiExample} />
      <DemoFrame title="Test runner">
        <div class="tests__panel mono">
          <p>Status is 200 · <span class="ok">pass</span> · 4 ms</p>
          <p>Body has user field · <span class="ok">pass</span> · 1 ms</p>
          <p>Response under 500ms · <span class="ok">pass</span></p>
        </div>
      </DemoFrame>
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <p class="eyebrow fade-up">It's a binary</p>
      <h2 class="display-2 fade-up stagger-1"><code class="mono ci-kicker">stubhouse serve</code> and ship.</h2>
      <p class="body-lg fade-up stagger-2">
        The same engine that runs in the desktop app runs from the command line. Spin up your mocks in CI. Run your assertion suite
        against them. Get JUnit XML out the other side. Ten kilobytes of YAML replaces a thousand lines of `nock`.
      </p>
    </div>
    <div class="fade-up stagger-2">
      <CodeBlock filename="ci.sh" language="bash" code={ciExample} />
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container">
    <p class="eyebrow fade-up">Honestly</p>
    <h2 class="display-2 fade-up stagger-1">Where StubHouse fits.</h2>
    <p class="body-lg prose-width fade-up stagger-2">
      A direct comparison with the tools you already know. We ship the embedded mock as part of the product, not as an upsell.
    </p>
    <div class="fade-up stagger-3">
      <Comparison />
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <p class="eyebrow fade-up">Under the hood</p>
      <h2 class="display-2 fade-up stagger-1">Native binary. No Electron.</h2>
      <p class="body-lg fade-up stagger-2">
        Tauri 2 shell. Rust core. `hyper` HTTP runtime. `rustls` for TLS. Embedded mock server runs in a Tokio task in the same
        process as the UI. Numbers below are illustrative until we publish measured builds beside shipping artifacts.
      </p>
    </div>
    <div class="fade-up stagger-2 chart" aria-label="Illustrative install size comparison">
      <div class="chart__row">
        <span class="mono chart__lab">StubHouse</span>
        <div class="chart__track"><span class="chart__bar chart__bar--solid" style="width: 22%"></span></div>
        <span class="caption chart__num">~25 MB</span>
      </div>
      <div class="chart__row">
        <span class="mono chart__lab">Insomnia</span>
        <div class="chart__track"><span class="chart__bar chart__bar--striped" style="width: 100%"></span></div>
        <span class="caption chart__num">~110 MB</span>
      </div>
      <div class="chart__row">
        <span class="mono chart__lab">Postman</span>
        <div class="chart__track"><span class="chart__bar chart__bar--striped" style="width: 100%"></span></div>
        <span class="caption chart__num larger">~350 MB</span>
      </div>
      <p class="caption chart__note">Cold-start bar uses the same discipline — measure before launch, publish with releases.</p>
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <p class="eyebrow fade-up">It's free. It's open.</p>
      <h2 class="display-2 fade-up stagger-1">MIT licensed.</h2>
      <p class="body-lg fade-up stagger-2">
        StubHouse is open source under the MIT License. No usage tracking. No telemetry without an explicit opt-in. No "Pro" features
        held hostage. The full source — desktop app, CLI, and core libraries — is on GitHub. PRs welcome.
      </p>
    </div>
    <div class="fade-up stagger-2">
      <CodeBlock filename="terminal" language="bash" code={terminalBuild} />
    </div>
  </div>
</SectionReveal>

<section class="final-cta">
  <div class="container final-cta__inner">
    <div>
      <p class="eyebrow">Ready when the API isn't</p>
      <h2 class="final-cta__title">Run real requests and local mocks from one native app.</h2>
      <p class="final-cta__sub">
        Free, open source, and available for macOS, Linux, and Windows.
      </p>
    </div>
    <div class="final-cta__actions">
      <Button href="/download" variant="primary" size="lg">
        {#snippet lead()}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
            <path d="M12 4v12m0 0 4-4m-4 4-4-4M4 20h16" />
          </svg>
        {/snippet}
        Download for macOS
      </Button>
      <Button href="https://github.com/stubhouse/stubhouse" variant="secondary" size="lg" target="_blank" rel="noopener noreferrer">
        View on GitHub
        {#snippet trailing()}<span aria-hidden="true">→</span>{/snippet}
      </Button>
    </div>
  </div>
</section>

<style>
  .hero {
    min-height: calc(100vh - var(--nav-h));
    display: flex;
    align-items: flex-start;
    padding-top: clamp(48px, 8vh, 88px);
    padding-bottom: clamp(48px, 8vh, 84px);
    position: relative;
    overflow: clip;
  }

  .hero::before {
    content: '';
    position: absolute;
    inset: auto 0 0;
    height: 42%;
    pointer-events: none;
    background: linear-gradient(to top, var(--bg-surface), transparent);
    opacity: 0.32;
  }

  @media (max-width: 639px) {
    .hero {
      min-height: auto;
      padding-top: 28px;
      padding-bottom: 36px;
    }
  }

  .hero__inner {
    position: relative;
    max-width: var(--max-hero);
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .hero__mark img {
    display: block;
    width: 88px;
    height: 88px;
    margin-inline: auto;
    border-radius: 18px;
    box-shadow:
      0 22px 70px rgba(0, 0, 0, 0.32),
      0 0 0 1px var(--border-subtle);
  }

  .hero__title {
    max-width: 11ch;
    margin-top: 12px;
  }

  .hero__sub {
    margin-top: 20px;
    max-width: 700px;
  }

  .hero__cta {
    margin-top: 32px;
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    justify-content: center;
  }

  .hero__platforms {
    margin: 14px 0 0;
  }

  .hero__proof {
    margin-top: 28px;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    color: var(--text-secondary);
    font-size: var(--text-body-sm);
  }

  .hero__proof span {
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 7px 11px;
    background: color-mix(in srgb, var(--bg-surface) 58%, transparent);
    box-shadow: inset 0 1px 0 var(--border-subtle);
  }

  .hero__proof strong {
    color: var(--text-primary);
    font-weight: 500;
  }

  .hero__demo {
    width: 100%;
    margin-top: clamp(36px, 7vh, 68px);
    text-align: left;
    transform-origin: top center;
  }

  .hero__demo-note {
    margin: 14px auto 0;
    max-width: 72ch;
    text-align: center;
  }

  @media (max-width: 639px) {
    .hero__inner {
      align-items: stretch;
      text-align: left;
      max-width: 460px;
    }

    .hero__mark {
      display: none;
    }

    .hero__mark img {
      width: 64px;
      height: 64px;
      margin-inline: 0;
      border-radius: 14px;
      box-shadow:
        0 16px 48px rgba(0, 0, 0, 0.28),
        0 0 0 1px var(--border-subtle);
    }

    .hero .eyebrow {
      margin-bottom: 10px;
    }

    .hero__title {
      max-width: none;
      margin-top: 0;
      text-wrap: balance;
    }

    .hero__sub {
      margin-top: 14px;
      max-width: 36ch;
    }

    .hero__cta {
      margin-top: 22px;
      display: grid;
      grid-template-columns: 1fr;
      gap: 8px;
      justify-content: stretch;
    }

    .hero__platforms {
      margin-top: 12px;
    }

    .hero__proof {
      margin-top: 18px;
      justify-content: flex-start;
      gap: 7px;
    }

    .hero__proof span {
      padding: 5px 8px;
      font-size: 11px;
    }

    .hero__demo {
      margin-top: 26px;
      min-width: 0;
    }

    .hero__demo-note {
      margin-top: 10px;
      text-align: left;
    }
  }

  @media (prefers-reduced-motion: no-preference) {
    .hero-ent {
      opacity: 0;
      transform: translateY(8px);
      animation: hero-in 400ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
    }
    .hero-ent--1 {
      animation-delay: 80ms;
    }
    .hero-ent--2 {
      animation-delay: 140ms;
    }
    .hero-ent--3 {
      animation-delay: 200ms;
    }
    .hero-ent--4 {
      animation-delay: 260ms;
    }
    .hero-ent--5 {
      animation-delay: 320ms;
    }
  }

  @keyframes hero-in {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .hero-ent {
      opacity: 1;
    }
  }

  .dual {
    display: grid;
    gap: 16px;
  }

  @media (max-width: 639px) {
    .dual {
      gap: 12px;
    }
  }

  .dual__panel {
    padding: 14px;
  }

  .dual__bar {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0 0 12px;
  }

  .dual__pre {
    margin: 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-primary);
    white-space: pre-wrap;
  }

  .dual__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 12px;
    color: var(--text-primary);
  }

  .dual__tag {
    margin-left: 8px;
    padding: 2px 8px;
    border: 1px solid var(--border-default);
    border-radius: 999px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .dual__hint {
    margin: 16px 0 0;
  }

  .mock-head {
    margin-bottom: clamp(40px, 8vw, 72px);
  }

  .mock-rows {
    display: flex;
    flex-direction: column;
    gap: clamp(56px, 10vw, 96px);
  }

  @media (max-width: 639px) {
    .mock-head {
      margin-bottom: 32px;
    }

    .mock-rows {
      gap: 48px;
    }
  }

  .mock-row {
    display: grid;
    gap: clamp(24px, 5vw, 48px);
    align-items: start;
    padding-block: clamp(8px, 2vw, 20px);
  }

  @media (min-width: 1024px) {
    .mock-row {
      grid-template-columns: 1fr 1fr;
    }
    .mock-row--flip :first-child {
      order: 2;
    }
    .mock-row--flip :last-child {
      order: 1;
    }
  }

  .curl-block {
    margin: 12px 0 0;
    padding: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    border-radius: var(--radius-md);
    font-size: 12px;
    line-height: 1.45;
    white-space: pre-wrap;
  }

  @media (max-width: 639px) {
    .curl-block {
      overflow-x: auto;
      white-space: pre;
      font-size: 11px;
    }
  }

  .fault-grid {
    margin-top: 40px;
    display: grid;
    gap: 12px;
  }

  @media (max-width: 639px) {
    .fault-grid {
      margin-top: 28px;
    }

    .fault-card__yaml {
      opacity: 1;
      max-height: none;
    }
  }

  @media (min-width: 768px) {
    .fault-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (min-width: 1100px) {
    .fault-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .fault-card {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: 16px;
    background:
      linear-gradient(to bottom, color-mix(in srgb, var(--bg-surface-2) 42%, transparent), transparent),
      var(--bg-surface);
    transition:
      border-color 200ms ease,
      transform 200ms var(--ease-out);
    min-height: 100%;
  }

  .fault-card:hover {
    border-color: var(--border-strong);
    transform: translateY(-2px);
  }

  .fault-card__name {
    margin: 0 0 8px;
    font-size: 13px;
  }

  .fault-card__hint {
    margin: 0 0 12px;
  }

  .fault-card__yaml {
    margin: 0;
    font-size: 11px;
    line-height: 1.45;
    color: var(--text-secondary);
    white-space: pre-wrap;
    opacity: 0;
    max-height: 0;
    overflow: hidden;
    transition:
      opacity 200ms ease,
      max-height 200ms ease;
  }

  .fault-card:hover .fault-card__yaml,
  .fault-card:focus-within .fault-card__yaml {
    opacity: 1;
    max-height: 120px;
  }

  .tests__panel {
    padding: 12px;
    font-size: 12px;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .automation-stack {
    display: grid;
    gap: 14px;
  }

  .tests__panel .ok {
    color: var(--text-primary);
  }

  .ci-kicker {
    font-size: inherit;
    letter-spacing: -0.03em;
  }

  .chart__row {
    display: grid;
    grid-template-columns: 104px 1fr 72px;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  @media (max-width: 639px) {
    .chart__row {
      grid-template-columns: 86px 1fr 58px;
      gap: 8px;
    }

    .chart__lab,
    .chart__num {
      font-size: 11px;
    }
  }

  .chart__track {
    position: relative;
    height: 10px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    overflow: hidden;
  }

  .chart__bar {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    max-width: 100%;
    background: var(--text-primary);
  }

  .chart__bar--striped {
    background: repeating-linear-gradient(
      -45deg,
      var(--text-primary),
      var(--text-primary) 2px,
      transparent 2px,
      transparent 5px
    );
    opacity: 0.35;
  }

  .chart__bar--solid {
    opacity: 1;
  }

  .chart__num {
    text-align: right;
    margin: 0;
  }

  .chart__num.larger {
    font-size: 11px;
  }

  .chart__note {
    margin-top: 16px;
  }

  .final-cta {
    padding-block: clamp(40px, 7vw, 72px);
    border-top: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
  }

  .final-cta__inner {
    display: grid;
    gap: 28px;
    align-items: center;
  }

  .final-cta__title {
    max-width: 760px;
    margin: 0;
    font-family: var(--font-sans);
    font-size: var(--text-h2, 28px);
    line-height: 1.18;
    letter-spacing: -0.02em;
    font-weight: 500;
  }

  .final-cta__sub {
    margin: 12px 0 0;
    max-width: 620px;
    color: var(--text-secondary);
    font-size: var(--text-body);
    line-height: 1.6;
  }

  .final-cta__actions {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  @media (max-width: 639px) {
    .final-cta {
      padding-block: 40px;
    }

    .final-cta__inner {
      gap: 22px;
    }

    .final-cta__title {
      font-size: 22px;
      line-height: 1.22;
    }

    .final-cta__actions {
      display: grid;
      grid-template-columns: 1fr;
      gap: 10px;
    }
  }

  @media (min-width: 900px) {
    .final-cta__inner {
      grid-template-columns: minmax(0, 1fr) auto;
    }

    .final-cta__actions {
      justify-content: flex-end;
    }
  }
</style>
