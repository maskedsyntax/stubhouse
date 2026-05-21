<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import SectionReveal from '$lib/SectionReveal.svelte';
  import DemoFrame from '$lib/components/DemoFrame.svelte';

  const meta = {
    title: 'The mock server',
    description: 'Embedded HTTP mocks, YAML rules, scenarios, logs, hot reload, faults, passthrough, recording, and stateful resources.'
  };

  const ruleYaml = `matcher:
  method: GET
  path: "/users/:id"
response:
  status: 200
  headers:
    content-type: application/json
  body:
    id: "{{params.id}}"
    name: "Alice"`;

  const runtime = [
    'Scenario model and switcher',
    'Desktop mock server panel with live request log',
    'Hot reload for mock YAML, resources, and recording config',
    'Control API under /__mirage/*',
    'Fault injection and selective passthrough',
    'Recording mode, scrub config, fixture capture, and stateful resources'
  ];

  const scenarioYaml = `name: get-user
method: GET
path: /users/:id
response:
  status: 200
  body:
    kind: json
    text: '{"id":"{{params.id}}","state":"default"}'
scenarios:
  - name: outage
    active: false
    response:
      status: 503
      body:
        kind: json
        text: '{"error":"maintenance"}'`;
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="page-shell section-tight container">
  <div class="page-hero-grid">
    <div>
      <p class="eyebrow">Mocks</p>
      <h1 class="display-2">The mock server is the product.</h1>
      <p class="body-lg prose-width">
        The mock runtime now includes YAML rules, a Rust parser, priority route matching, `stubhouse serve`, scenarios, hot reload,
        live logs, faults, passthrough, recording, and stateful resource helpers.
      </p>
      <div class="status-strip" aria-label="Mock runtime status">
        <span class="status-pill">CLI serve</span>
        <span class="status-pill">Desktop panel</span>
        <span class="status-pill">Scenarios</span>
        <span class="status-pill">Hot reload</span>
      </div>
    </div>
    <CodeBlock filename="scenario.yaml" language="yaml" code={scenarioYaml} />
  </div>
</section>

<SectionReveal class="section">
  <div class="container">
    <h2 class="display-3 fade-up">Architecture</h2>
    <p class="body-lg prose-width fade-up stagger-1">
      The mock server is built on Hyper and Tokio in the Rust core. It binds locally from the CLI or app, loads workspace rules,
      emits request logs, reloads valid edits in place, and matches routes with exact paths, path params, wildcards, and catch-alls.
    </p>
    <DemoFrame title="Diagram — runtime" class="fade-up stagger-2">
      <div class="diagram mono">
        <pre>{`┌──────────── Svelte UI ────────────┐
│  request editor · mock panel · log  │
└───────────────┬────────────────────┘
                │ IPC
┌───────────────▼────────────────────┐
│            Rust core                │
│  request engine │ mock hyper server │
│  config · Rhai  · vars · history    │
└────────────────────────────────────┘`}</pre>
      </div>
    </DemoFrame>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container grid-2">
    <div>
      <h2 class="display-3 fade-up">Rule anatomy</h2>
      <p class="body-lg fade-up stagger-1">
        A rule is a matcher plus a response. Scenario variants will sit under the same matcher later; the current implementation keeps
        the shape simple while the runtime and matcher settle.
      </p>
    </div>
    <div class="fade-up stagger-2">
      <CodeBlock filename="rules.yaml" language="yaml" code={ruleYaml} />
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container">
    <h2 class="display-3 fade-up">Runtime surface</h2>
    <p class="body-lg prose-width fade-up stagger-1">
      The current phase is focused on making those mock features easier to inspect, compose, and document from the desktop workflow.
    </p>
    <ul class="mono table-like fade-up stagger-2">
      {#each runtime as item}
        <li>{item}</li>
      {/each}
    </ul>
  </div>
</SectionReveal>

<style>
  .diagram pre {
    margin: 0;
    padding: 16px;
    font-size: 11px;
    line-height: 1.45;
    color: var(--text-secondary);
    overflow: auto;
  }

  .table-like {
    margin-top: 24px;
    padding: 16px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .table-like li + li {
    margin-top: 8px;
  }
</style>
