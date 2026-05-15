<script lang="ts">
  import type { PageProps } from './$types';

  const bodies: Record<string, string> = {
    'route-trie-in-rust': `The mock server sees every request as a tuple: method, path segments, headers, body. We wanted first-match semantics with predictable precedence — exact segments before parameters before globs — without paying for a regex explosion on every call.

We ended on a priority trie keyed by segment index. Each node stores the match kinds that could end a route at that depth, along with rule priority for tie-breaks. Wildcards collapse to a single branch; parameters are preserved for the response renderer and the planned scripting layer.

Hot reload is next: the goal is to rebuild only the touched branchlist so YAML edits from your editor stay cheap. This is the sort of structure that is boring when it works, and obvious only after you try the alternatives.

Next posts will cover fault injection hooks and how we keep Rhai scripts from touching the network.`,
    'rhai-over-v8': `Postman proved developers want script hooks. They also shipped a multi-megabyte runtime to do it.

StubHouse is allergic to that trade. The planned Rhai layer is designed around a sandbox without filesystem or sockets, fast startup, and a binary budget that still makes sense for a dock icon you leave open all day.

The language is small on purpose: enough to sign requests, assert responses, and branch mocks — not to run npm install inside your API client.

We will publish benches alongside releases. The goal is not to win a shootout with Lua; it is to keep the security story boring.`
  };

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{data.title} — StubHouse</title>
  <meta name="description" content={data.description} />
</svelte:head>

<article class="container article">
  <p class="caption"><a class="link-inline" href="/blog">Blog</a></p>
  <h1 class="display-2">{data.title}</h1>
  <p class="caption meta">{data.date} · {data.read}</p>
  {#each bodies[data.slug].split('\n\n') as para}
    <p class="body-lg prose">{para}</p>
  {/each}
</article>

<style>
  .article {
    max-width: 720px;
    padding-block: 64px;
  }

  .meta {
    margin-bottom: 32px;
  }

  .prose {
    margin: 0 0 20px;
    max-width: 65ch;
  }
</style>
