import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch }) => {
  let githubStars: number | null = null;
  try {
    const res = await fetch('https://api.github.com/repos/stubhouse/stubhouse', {
      headers: { Accept: 'application/vnd.github+json' }
    });
    if (res.ok) {
      const j = (await res.json()) as { stargazers_count?: number };
      if (typeof j.stargazers_count === 'number') githubStars = j.stargazers_count;
    }
  } catch {
    githubStars = null;
  }
  return { githubStars };
};
