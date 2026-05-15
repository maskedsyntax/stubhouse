<svelte:head>
  <title>Manifesto — StubHouse</title>
  <meta
    name="description"
    content="Why StubHouse argues for local-first API tooling, files as source of truth, and mocks as craft — not concession."
  />
</svelte:head>

<section class="container manifesto">
  <h1 class="manifesto__title">Calm tools for noisy networks.</h1>

  <p>
    StubHouse exists because working without a backend is ordinary work. Teams ship front-ends first, vendors slip deadlines, and
    sandboxes fall over the moment you depend on them. The API client on your desk should acknowledge that reality instead of
    pretending every endpoint is always reachable.
  </p>

  <p>
    Local-first is not a posture statement. It is permission to close the laptop on a plane and still merge a feature branch. When
    your definitions live in plaintext next to your product code, reviews read like reviews — not like archaeology.
  </p>

  <p>
    Mocking is often treated as a concession: a stopgap until the “real” service appears. We treat it as part of the craft. A mock
    runtime that grows into scenarios, state, and an honest control surface becomes a design tool. It lets you rehearse failures you
    cannot yet buy from an upstream team.
  </p>

  <p>
    The desktop should feel native because attention is finite. Electron served a generation; it should not be the forever default for
    tools you touch hundreds of times a day. A quiet binary that starts fast and respects the OS theme is table stakes.
  </p>

  <p>
    We are not building a platform narrative. StubHouse is a sharp instrument — request sender, embedded server, files on disk. If
    we ever add a hosted layer, it must earn its place by reducing friction you already feel, not by inventing collaboration theater.
  </p>

  <p class="caption signoff">— manifesto draft v0.1</p>
</section>

<style>
  .manifesto {
    max-width: 640px;
    padding-block: 96px 120px;
    font-family: var(--font-serif);
    font-size: 20px;
    line-height: 1.65;
    color: var(--text-secondary);
  }

  .manifesto__title {
    font-family: var(--font-serif);
    font-weight: 400;
    font-size: var(--text-display-3);
    line-height: 1.2;
    color: var(--text-primary);
    margin: 0 0 32px;
    letter-spacing: 0;
  }

  .manifesto p {
    margin: 0 0 24px;
  }

  .signoff {
    margin-top: 48px;
    font-family: var(--font-sans);
  }
</style>
