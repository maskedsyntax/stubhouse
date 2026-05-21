<script lang="ts">
  import CodeBlock from '$lib/components/CodeBlock.svelte';

  const meta = {
    title: 'Documentation',
    description:
      'Complete StubHouse documentation for installation, workspaces, requests, environments, mocks, CLI automation, scripting, and tests.'
  };

  const quickstart = `git clone https://github.com/stubhouse/stubhouse
cd stubhouse
cd src-frontend
bun install
bun run tauri dev

# in another shell, from the repo root
cargo run -p stubhouse-cli -- --help`;

  const workspaceTree = `acme-api/
  workspace.yaml
  environments/
    mock.yaml
    prod.yaml
  collections/
    users/
      get-user.yaml
      create-user.yaml
      mocks/
        get-user.yaml
    billing/
      list-invoices.yaml
  fixtures/
    users.yaml
  recordings/`;

  const workspaceYaml = `name: Acme API
version: "1"
mock_resources:
  - path: /api/users
    id_field: id
    seed_file: fixtures/users.yaml
    auto_crud: true
recording:
  dir: recordings
  scrub:
    replacement: "[REDACTED]"
    headers: [authorization, x-api-key]
    json_fields: [password, token, secret]
    text: [secret-token]`;

  const requestYaml = `name: Get user
description: Fetch a single user by id.
method: GET
url: "{{base_url}}/api/users/{{user_id}}"
query:
  - [include, teams]
headers:
  - [Accept, application/json]
auth:
  kind: bearer
  token: "{{api_token}}"
body:
  kind: none
post_response_script: |
  test("status is 200") { response.status == 200 }
  test("fast enough") { response.elapsed_ms < 500 }`;

  const envYaml = `name: mock
variables:
  base_url: http://127.0.0.1:4000
  user_id: u_42
secrets:
  api_token:
    source: env
    var: STUBHOUSE_API_TOKEN`;

  const mockRule = `name: get-user
method: GET
path: /api/users/:id
priority: 20
response:
  status: 200
  headers:
    - [Cache-Control, no-store]
  body:
    kind: json
    text: |
      {"id":"{{params.id}}","name":"Ada","plan":"team"}
scenarios:
  - name: outage
    active: false
    response:
      status: 503
      body:
        kind: json
        text: '{"error":"maintenance"}'
fault:
  kind: slow_response
  delay_ms: 900
condition_script: request.params["id"] != "blocked"`;

  const passthroughRule = `name: proxy-users
method: GET
path: /api/proxy/**
priority: 1
passthrough: true
upstream_url: https://api.example.com
record: true`;

  const bodyScript = `name: generated-user
method: GET
path: /api/generated/:id
response:
  status: 200
  body_script: |
    \`{"id":"\${request.params["id"]}","scripted":true}\``;

  const cli = `stubhouse init acme-api
stubhouse validate
stubhouse list
stubhouse show collections/users/get-user.yaml
stubhouse envs

stubhouse import postman ./collection.json
stubhouse import openapi ./openapi.yaml
stubhouse export curl collections/users/get-user.yaml --env mock
stubhouse export openapi --output openapi.yaml

stubhouse serve --port 4000
stubhouse scenario list
stubhouse scenario activate outage
stubhouse test --env mock --junit results.xml`;

  const controlApi = `curl http://127.0.0.1:4000/__mirage/status
curl http://127.0.0.1:4000/__mirage/rules
curl http://127.0.0.1:4000/__mirage/log?limit=20
curl -X POST http://127.0.0.1:4000/__mirage/scenario \\
  -H 'content-type: application/json' \\
  -d '{"scenario":"outage"}'
curl -X POST http://127.0.0.1:4000/__mirage/reset`;

  const scripts = `pre_request_script: |
  request.headers["X-Trace"] = env["trace_id"];
  request.query["debug"] = "true";

post_response_script: |
  test("status is 200") { response.status == 200 }
  test("body contains id") { response.body.contains("u_42") }
  variables["last_status"] = response.status.to_string();`;

  const installSteps = [
    ['Clone', 'Clone the repo and run the desktop shell from src-frontend.'],
    ['Open', 'Open a folder with workspace.yaml, or let the app initialize one.'],
    ['Send', 'Create a request, choose an environment, and send it.'],
    ['Mock', 'Add YAML rules under collections/<name>/mocks and start the mock server.'],
    ['Automate', 'Use the CLI for validation, imports, exports, serving mocks, scenarios, and tests.']
  ];

  const sections = [
    ['getting-started', 'Getting Started'],
    ['workspace', 'Workspace Layout'],
    ['requests', 'Requests'],
    ['environments', 'Environments'],
    ['cli', 'CLI'],
    ['mocks', 'Mocks'],
    ['control-api', 'Control API'],
    ['scripting-tests', 'Scripting And Tests'],
    ['import-export', 'Import And Export'],
    ['troubleshooting', 'Troubleshooting']
  ];

  const troubleshoot = [
    ['workspace manifest not found', 'Run stubhouse init <name> in the workspace root, or open the directory that contains workspace.yaml.'],
    ['environment not found', 'Environment names come from environments/<name>.yaml. Check the file name and active workspace.'],
    ['invalid URL', 'Resolved request URLs must be absolute. Set base_url in the active environment before sending.'],
    ['invalid JSON body', 'JSON bodies are validated before send. Fix syntax or switch the body kind to text.'],
    ['bind 127.0.0.1:4000 failed', 'Another process is using the port. Start serve with --port 4001 or stop the other process.'],
    ['mock reload failed', 'The server keeps the last valid rules active. Fix the YAML error and save again.']
  ];
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section page-shell">
  <div class="docs-hero">
    <div>
      <p class="eyebrow">Docs</p>
      <h1 class="display-2">StubHouse documentation</h1>
      <p class="body-lg prose-width">
        Everything here describes the current source-built app and CLI: local workspaces, requests, environments, mocks, scenarios,
        recording, resources, scripting, imports, exports, and tests.
      </p>
    </div>
    <nav class="docs-toc surface-panel" aria-label="Documentation sections">
      {#each sections as [href, label]}
        <a href={`#${href}`}>{label}</a>
      {/each}
    </nav>
  </div>

  <section id="getting-started" class="doc-section">
    <div class="doc-copy">
      <p class="eyebrow">Getting started</p>
      <h2 class="display-3">Run the app from source.</h2>
      <p class="body-lg">
        Packaged installers are not published yet. The current development path is source-first: run the Tauri desktop app and use
        the Rust CLI from the repository root.
      </p>
    </div>
    <CodeBlock filename="terminal" language="bash" code={quickstart} />
    <div class="step-grid">
      {#each installSteps as [title, copy]}
        <article class="feature-card">
          <h3 class="feature-card__title">{title}</h3>
          <p class="feature-card__copy">{copy}</p>
        </article>
      {/each}
    </div>
  </section>

  <section id="workspace" class="doc-section doc-grid">
    <div class="doc-copy">
      <p class="eyebrow">Workspace layout</p>
      <h2 class="display-3">A workspace is a folder, not an account.</h2>
      <p class="body-lg">
        StubHouse opens a directory containing `workspace.yaml`. Requests live under `collections/`, environment files live under
        `environments/`, and mocks live beside the collection they describe.
      </p>
    </div>
    <CodeBlock filename="workspace tree" language="text" code={workspaceTree} />
    <CodeBlock filename="workspace.yaml" language="yaml" code={workspaceYaml} />
  </section>

  <section id="requests" class="doc-section doc-grid">
    <div class="doc-copy">
      <p class="eyebrow">Requests</p>
      <h2 class="display-3">Requests are canonical YAML files.</h2>
      <p class="body-lg">
        Request definitions combine display metadata with the HTTP compose model. Supported auth modes are `none`, `bearer`, `basic`,
        and `api_key`. Supported body kinds are `none`, `text`, `json`, and `form`.
      </p>
    </div>
    <CodeBlock filename="collections/users/get-user.yaml" language="yaml" code={requestYaml} />
    <div class="surface-panel doc-table">
      <div><strong>method</strong><span>GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS, TRACE.</span></div>
      <div><strong>url</strong><span>Absolute URL after interpolation.</span></div>
      <div><strong>query</strong><span>List of `[key, value]` pairs appended to the URL.</span></div>
      <div><strong>headers</strong><span>List of `[key, value]` pairs sent with the request.</span></div>
      <div><strong>auth</strong><span>Adds bearer, basic, or API key credentials.</span></div>
      <div><strong>body</strong><span>JSON is validated; form bodies are URL encoded.</span></div>
    </div>
  </section>

  <section id="environments" class="doc-section doc-grid">
    <div class="doc-copy">
      <p class="eyebrow">Environments</p>
      <h2 class="display-3">Variables resolve before send, export, and tests.</h2>
      <p class="body-lg">
        Use double-brace placeholders such as `base_url` in URLs, headers, auth tokens, and bodies. Plain variables are stored in YAML. Secrets can be
        loaded from OS environment variables and are merged into the active environment at runtime.
      </p>
    </div>
    <CodeBlock filename="environments/mock.yaml" language="yaml" code={envYaml} />
  </section>

  <section id="cli" class="doc-section">
    <div class="doc-copy">
      <p class="eyebrow">CLI</p>
      <h2 class="display-3">Use the same workspace in scripts and CI.</h2>
      <p class="body-lg">
        The CLI defaults to the current directory. Pass `--workspace /path/to/workspace` from anywhere to target a specific folder.
      </p>
    </div>
    <CodeBlock filename="terminal" language="bash" code={cli} />
  </section>

  <section id="mocks" class="doc-section doc-grid">
    <div class="doc-copy">
      <p class="eyebrow">Mocks</p>
      <h2 class="display-3">Rules, scenarios, faults, passthrough, and resources.</h2>
      <p class="body-lg">
        Mock rules are loaded from `collections/*/mocks/*.yaml`. Higher priority rules win. Paths support exact segments, `:params`,
        `*` wildcards, and `**` catch-alls. The server hot-reloads valid changes and keeps the last good rules when a YAML edit fails.
      </p>
    </div>
    <CodeBlock filename="collections/users/mocks/get-user.yaml" language="yaml" code={mockRule} />
    <CodeBlock filename="collections/users/mocks/proxy.yaml" language="yaml" code={passthroughRule} />
    <CodeBlock filename="collections/users/mocks/generated.yaml" language="yaml" code={bodyScript} />
  </section>

  <section id="control-api" class="doc-section">
    <div class="doc-copy">
      <p class="eyebrow">Control API</p>
      <h2 class="display-3">Drive mocks from tests.</h2>
      <p class="body-lg">
        Every running mock server exposes `__mirage` endpoints for status, rules, request logs, scenario switching, and state reset.
        Scenario switching through the control API changes in-memory rules; `stubhouse scenario activate` writes active flags to YAML.
      </p>
    </div>
    <CodeBlock filename="terminal" language="bash" code={controlApi} />
  </section>

  <section id="scripting-tests" class="doc-section doc-grid">
    <div class="doc-copy">
      <p class="eyebrow">Scripting and tests</p>
      <h2 class="display-3">Rhai handles request hooks and assertions.</h2>
      <p class="body-lg">
        Scripts run in a small sandbox with operation, call, map, array, and string limits. Pre-request scripts can mutate request
        fields. Post-response scripts can set variables and define assertion blocks. `stubhouse test` runs those
        assertions and can write JUnit XML.
      </p>
    </div>
    <CodeBlock filename="request scripts" language="rhai" code={scripts} />
  </section>

  <section id="import-export" class="doc-section">
    <div class="doc-copy">
      <p class="eyebrow">Import and export</p>
      <h2 class="display-3">Move in and out of common formats.</h2>
      <p class="body-lg">
        StubHouse imports Postman Collection v2.1 and OpenAPI 3 JSON/YAML into workspace request files. It exports individual
        requests as cURL and can emit an OpenAPI YAML document for the workspace.
      </p>
    </div>
    <div class="feature-grid feature-grid--3">
      <article class="feature-card">
        <h3 class="feature-card__title">Postman import</h3>
        <p class="feature-card__copy">Creates request YAML from a v2.1 collection file.</p>
      </article>
      <article class="feature-card">
        <h3 class="feature-card__title">OpenAPI import/export</h3>
        <p class="feature-card__copy">Reads OpenAPI 3 JSON/YAML and writes workspace OpenAPI YAML.</p>
      </article>
      <article class="feature-card">
        <h3 class="feature-card__title">cURL export</h3>
        <p class="feature-card__copy">Applies an optional environment and prints a runnable cURL command.</p>
      </article>
    </div>
  </section>

  <section id="troubleshooting" class="doc-section">
    <div class="doc-copy">
      <p class="eyebrow">Troubleshooting</p>
      <h2 class="display-3">Common errors.</h2>
    </div>
    <div class="surface-panel doc-table">
      {#each troubleshoot as [problem, fix]}
        <div><strong>{problem}</strong><span>{fix}</span></div>
      {/each}
    </div>
  </section>
</section>

<style>
  .docs-hero {
    display: grid;
    gap: clamp(28px, 5vw, 56px);
    align-items: start;
  }

  @media (min-width: 980px) {
    .docs-hero {
      grid-template-columns: minmax(0, 1fr) 300px;
    }
  }

  .docs-toc {
    position: sticky;
    top: calc(var(--nav-h) + 20px);
    display: grid;
    gap: 2px;
    padding: 10px;
  }

  .docs-toc a {
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    text-decoration: none;
    font-size: var(--text-body-sm);
  }

  .docs-toc a:hover,
  .docs-toc a:focus-visible {
    color: var(--text-primary);
    background: var(--shimmer);
  }

  .doc-section {
    scroll-margin-top: calc(var(--nav-h) + 24px);
    margin-top: clamp(56px, 9vw, 96px);
    padding-top: clamp(32px, 5vw, 56px);
    border-top: 1px solid var(--border-subtle);
  }

  .doc-grid {
    display: grid;
    gap: 18px;
  }

  @media (min-width: 980px) {
    .doc-grid {
      grid-template-columns: minmax(0, 0.78fr) minmax(0, 1fr);
      align-items: start;
    }

    .doc-grid .doc-copy {
      grid-column: 1;
      grid-row: 1 / span 12;
      position: sticky;
      top: calc(var(--nav-h) + 20px);
    }

    .doc-grid > :global(.code-block),
    .doc-grid > .doc-table {
      grid-column: 2;
    }
  }

  .doc-copy {
    max-width: 68ch;
  }

  .doc-copy .body-lg {
    margin-top: 14px;
  }

  .step-grid {
    margin-top: 18px;
    display: grid;
    gap: 12px;
  }

  @media (min-width: 760px) {
    .step-grid {
      grid-template-columns: repeat(5, minmax(0, 1fr));
    }
  }

  .doc-table {
    overflow: hidden;
  }

  .doc-table div {
    display: grid;
    gap: 6px;
    padding: 14px 16px;
  }

  .doc-table div + div {
    border-top: 1px solid var(--border-subtle);
  }

  @media (min-width: 700px) {
    .doc-table div {
      grid-template-columns: 180px minmax(0, 1fr);
      gap: 18px;
    }
  }

  .doc-table strong {
    color: var(--text-primary);
    font-weight: 500;
    font-size: var(--text-body-sm);
  }

  .doc-table span {
    color: var(--text-secondary);
    font-size: var(--text-body-sm);
    line-height: 1.55;
  }

  :global(.doc-section .code-block + .code-block) {
    margin-top: 16px;
  }

  @media (max-width: 639px) {
    .docs-toc {
      position: static;
    }

    .step-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
