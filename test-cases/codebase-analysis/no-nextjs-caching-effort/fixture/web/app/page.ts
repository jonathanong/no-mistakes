// Literal findings must retain their max messages under every low policy.
fetch('/literal', { cache: 'force-cache' });
fetch('/template', { cache: `force-cache` });
fetch('/next', { next: { revalidate: 60, tags: ['cache'] } });
const mode = 'force-cache';
fetch('/indirect', { cache: mode });
const duration = 60;
export const revalidate = duration;
export const dynamic = 'force-static';
fetch('/uncached', { cache: 'no-store', next: { revalidate: 0 } });
// no-mistakes-disable-next-line nextjs-no-caching
fetch('/suppressed', { cache: mode });
