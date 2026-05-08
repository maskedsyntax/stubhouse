import { error } from '@sveltejs/kit';
import type { EntryGenerator, PageLoad } from './$types';

export const prerender = true;

export const entries: EntryGenerator = () => [
  { slug: 'route-trie-in-rust' },
  { slug: 'rhai-over-v8' }
];

export const load: PageLoad = ({ params }) => {
  const map: Record<string, { title: string; description: string; date: string; read: string }> = {
    'route-trie-in-rust': {
      title: 'How we built a route trie matcher in Rust',
      description: 'Prefix trees, parameters, and first-match semantics for the StubHouse mock server.',
      date: '2026-05-06',
      read: '8 min read'
    },
    'rhai-over-v8': {
      title: 'Why Rhai instead of an embedded V8',
      description: 'Sandboxing, binary size, and the scripting surface for StubHouse.',
      date: '2026-05-02',
      read: '6 min read'
    }
  };

  const m = map[params.slug];
  if (!m) error(404, 'Not found');
  return { slug: params.slug, ...m };
};
