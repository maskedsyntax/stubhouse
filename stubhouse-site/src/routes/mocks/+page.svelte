<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import SectionReveal from '$lib/SectionReveal.svelte';
  import DemoFrame from '$lib/components/DemoFrame.svelte';

  const meta = {
    title: 'The mock server',
    description: 'Embedded HTTP mocks, scenarios, state, recording, fault injection, and a control API built for real workflows.'
  };

  const ruleYaml = `matcher:
  method: GET
  path: "/users/:id"
scenarios:
  - name: success
    active: true
    response:
      status: 200
      body:
        id: "{{params.id}}"
        name: "Alice"
  - name: not_found
    response:
      status: 404
      body:
        error: "User not found"`;
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="page-hero section-tight container">
  <p class="eyebrow">Mocks</p>
  <h1 class="display-2">The mock server is the product.</h1>
  <p class="body-lg prose-width">
    This page is the long-form treatment of what the home page previews: first-class rules, scenarios you can flip without restarting,
    state that behaves like a service, recording, passthrough, and faults you can turn on like switches.
  </p>
</section>

<SectionReveal class="section">
  <div class="container">
    <h2 class="display-3 fade-up">Architecture</h2>
    <p class="body-lg prose-width fade-up stagger-1">
      The mock server is an embedded Tokio task beside the UI. It binds locally, matches with a priority trie, evaluates Rhai where
      you need logic, and streams events back to the panels you already have open.
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
        A rule is a matcher plus responses. Scenarios sit under the same matcher so you can rehearse success, errors, and slow paths
        without duplicating routes.
      </p>
    </div>
    <div class="fade-up stagger-2">
      <CodeBlock filename="rules.yaml" language="yaml" code={ruleYaml} />
    </div>
  </div>
</SectionReveal>

<SectionReveal class="section">
  <div class="container">
    <h2 class="display-3 fade-up">Control API</h2>
    <p class="body-lg prose-width fade-up stagger-1">
      Everything under `/__mirage/` is there for your tests: status, scenarios, resets, logs, rule dumps, fault toggles.
    </p>
    <pre class="mono table-like fade-up stagger-2">GET  /__mirage/status
POST /__mirage/scenario
POST /__mirage/reset
GET  /__mirage/rules
GET  /__mirage/log
POST /__mirage/fault</pre>
  </div>
</SectionReveal>

<style>
  .page-hero {
    padding-top: 48px;
  }

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
    background: var(--bg-surface);
    font-size: 13px;
    line-height: 1.6;
    white-space: pre-wrap;
  }
</style>
