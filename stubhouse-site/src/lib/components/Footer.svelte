<footer class="footer">
  <div class="container footer__grid">
    <div class="footer__brand">
      <a href="/" class="footer__logo" aria-label="StubHouse home">
        <img src="/stubhouse-logo-rounded-192.png" width="48" height="48" alt="" />
      </a>
      <p class="footer__tag caption">
        A local-first API client and mock server. Native, offline, file-based.
      </p>
    </div>
    <div>
      <p class="footer__col-title caption">Product</p>
      <ul class="footer__list">
        <li><a href="/mocks">Mocks</a></li>
        <li><a href="/pricing">Pricing</a></li>
        <li><a href="/download">Download</a></li>
      </ul>
    </div>
    <div>
      <p class="footer__col-title caption">Resources</p>
      <ul class="footer__list">
        <li><a href="/docs">Docs</a></li>
        <li><a href="https://github.com/stubhouse/stubhouse">GitHub</a></li>
        <li><a href="/changelog">Changelog</a></li>
        <li>
          <a href="https://github.com/stubhouse/stubhouse/blob/main/spec.md" target="_blank" rel="noopener noreferrer">Roadmap</a>
        </li>
      </ul>
    </div>
    <div>
      <p class="footer__col-title caption">Company</p>
      <ul class="footer__list">
        <li><a href="/manifesto">Manifesto</a></li>
        <li><a href="/blog">Blog</a></li>
        <li><a href="/press">Press</a></li>
      </ul>
    </div>
    <div>
      <p class="footer__col-title caption">Legal</p>
      <ul class="footer__list">
        <li><a href="/legal/privacy">Privacy</a></li>
        <li><a href="/legal/terms">Terms</a></li>
        <li><a href="/legal/license">License</a></li>
      </ul>
    </div>
  </div>
  <hr class="divider footer__rule" />
  <div class="container footer__bottom">
    <p class="caption">© 2026 StubHouse · MIT</p>
    <p class="caption">Built in Rust and Svelte.</p>
    <p class="caption footer__analytics">
      Plausible Analytics ·
      <a class="link-inline" href="/legal/privacy">No cookies. No personal data.</a>
    </p>
  </div>
</footer>

<style>
  .footer {
    padding-block: 64px 48px;
  }

  .footer__grid {
    display: grid;
    gap: 40px;
  }

  @media (min-width: 900px) {
    .footer__grid {
      grid-template-columns: 1.2fr repeat(4, 1fr);
      align-items: start;
    }
  }

  @media (max-width: 639px) {
    .footer {
      padding-block: 44px 32px;
    }

    .footer__grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 28px 20px;
    }

    .footer__brand {
      grid-column: 1 / -1;
    }
  }

  .footer__logo {
    display: inline-flex;
    color: var(--text-primary);
    margin-bottom: 12px;
  }

  .footer__logo img {
    border-radius: 10px;
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.2);
  }

  .footer__tag {
    max-width: 28ch;
    margin: 0;
  }

  .footer__col-title {
    margin: 0 0 12px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .footer__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .footer__list a {
    color: var(--text-secondary);
    text-decoration: none;
    font-size: var(--text-body-sm);
    transition: color 150ms ease;
  }

  .footer__list a:hover,
  .footer__list a:focus-visible {
    color: var(--text-primary);
  }

  .footer__rule {
    margin: 48px 0 24px;
  }

  @media (max-width: 639px) {
    .footer__rule {
      margin: 32px 0 20px;
    }
  }

  .footer__bottom {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  @media (min-width: 768px) {
    .footer__bottom {
      flex-direction: row;
      flex-wrap: wrap;
      gap: 16px 32px;
      align-items: center;
    }
    .footer__bottom p {
      margin: 0;
    }
    .footer__analytics {
      margin-left: auto;
    }
  }
</style>
