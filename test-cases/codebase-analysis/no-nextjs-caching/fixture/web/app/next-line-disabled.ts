export async function load() {
  // no-mistakes-disable-next-line nextjs-no-caching
  return fetch('/api/a', { cache: 'force-cache' })
}
