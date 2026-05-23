export async function load() {
  // guardrails-disable-next-line nextjs-no-caching
  return fetch('/api/a', { cache: 'force-cache' })
}
