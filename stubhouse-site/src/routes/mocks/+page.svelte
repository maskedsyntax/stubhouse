<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';
  import SectionReveal from '$lib/SectionReveal.svelte';
  import DemoFrame from '$lib/components/DemoFrame.svelte';

  const meta = {
    title: 'The mock server',
    description: 'Embedded HTTP mocks, YAML rules, priority route matching, and the roadmap for scenarios, logs, faults, and recording.'
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

  const roadmap = [
    'Scenario model and switcher',
    'Desktop mock server panel with live request log',
    'Hot reload for mock YAML',
    'Control API under /__mirage/*',
    'Fault injection and selective passthrough',
    'Recording mode and fixture capture'
  ];
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="page-hero section-tight container">
  <p class="eyebrow">Mocks</p>
  <h1 class="display-2">The mock server is the product.</h1>
  <p class="body-lg prose-width">
    The core mock runtime is now in place: YAML rules, a Rust parser, a priority trie matcher, and `stubhouse serve` for headless
    local APIs. The desktop panel, scenarios, hot reload, faults, passthrough, and recording are the next layer.
  </p>
</section>

<SectionReveal class="section">
  <div class="container">
    <h2 class="display-3 fade-up">Architecture</h2>
    <p class="body-lg prose-width fade-up stagger-1">
      The mock server is built on Hyper and Tokio in the Rust core. Today it binds locally from the CLI, loads workspace mock rules,
      and matches routes with exact paths, path params, wildcards, and catch-alls.
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
    <h2 class="display-3 fade-up">Next up</h2>
    <p class="body-lg prose-width fade-up stagger-1">
      Phase 2 is focused on making the mock runtime interactive from the desktop app and controllable from tests.
    </p>
    <ul class="mono table-like fade-up stagger-2">
      {#each roadmap as item}
        <li>{item}</li>
      {/each}
    </ul>
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
    color: var(--text-secondary);
  }

  .table-like li + li {
    margin-top: 8px;
  }
</style>
