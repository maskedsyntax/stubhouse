# StubHouse Kitchen Sink Workspace

Open this directory from the StubHouse app:

```text
examples/workspaces/kitchen-sink
```

Suggested UI test flow:

1. Open the workspace.
2. Select the `mock` environment.
3. Start the mock server on `127.0.0.1:4000`.
4. Send requests from the `catalog`, `orders`, `auth`, and `mock-server` collections.
5. Switch scenarios between `happy_path`, `empty`, `error`, and `slow`.
6. Use the test results panel after the mock server is running.

This workspace covers request loading, environment interpolation, resolved URL preview, auth modes, JSON/text/form bodies, copy as cURL, mock rules, mock scenarios, route params, Rhai mock conditions, generated mock bodies, stateful CRUD resources, fixtures, recording config, request history, and collection tests.
