<script lang="ts">
  const posts = [
    {
      slug: 'route-trie-in-rust',
      title: 'How we built a route trie matcher in Rust',
      date: '2026-05-06',
      read: '8 min',
      excerpt: 'Prefix trees, parameter segments, and why first-match semantics beat fancy backtracking for local mocks.'
    },
    {
      slug: 'rhai-over-v8',
      title: 'Why Rhai instead of an embedded V8',
      date: '2026-05-02',
      read: '6 min',
      excerpt: 'Sandbox boundaries, binary weight, and the scripting surface we want for pre-request logic.'
    }
  ];

  const meta = {
    title: 'Engineering blog',
    description: 'Notes from building StubHouse — sparse, technical, and shipped when they are ready.'
  };
</script>

<svelte:head>
  <title>{meta.title} — StubHouse</title>
  <meta name="description" content={meta.description} />
</svelte:head>

<section class="container section">
  <p class="eyebrow">Blog</p>
  <h1 class="display-2">Writing</h1>
  <p class="body-lg prose-width">Long-form posts stay technical. No content calendar — only pieces worth finishing.</p>

  <ul class="posts">
    {#each posts as p}
      <li>
        <a class="post link-inline" href={`/blog/${p.slug}`}>
          <h2 class="post__title">{p.title}</h2>
          <p class="caption post__meta">{p.date} · {p.read}</p>
          <p class="body-lg post__excerpt">{p.excerpt}</p>
        </a>
      </li>
    {/each}
  </ul>
</section>

<style>
  .posts {
    list-style: none;
    margin: 48px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .post {
    display: block;
    text-decoration: none;
    color: inherit;
  }

  .post__title {
    margin: 0;
    font-size: 22px;
    font-weight: 500;
    letter-spacing: -0.015em;
  }

  .post__meta {
    margin: 8px 0;
  }

  .post__excerpt {
    margin: 0;
    max-width: 62ch;
  }

  .post:hover .post__title,
  .post:focus-visible .post__title {
    text-decoration: underline;
    text-underline-offset: 4px;
  }
</style>
