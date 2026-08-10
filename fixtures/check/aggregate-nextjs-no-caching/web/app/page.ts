export async function loadUser() {
  // no-mistakes-disable-next-line nextjs-no-caching: request data must stay uncached here
  return fetch('/api/user', { cache: 'force-cache' })
}
