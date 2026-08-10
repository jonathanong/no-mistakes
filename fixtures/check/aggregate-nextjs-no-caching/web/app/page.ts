export async function loadUser() {
  // no-mistakes-disable-next-line nextjs-no-caching: force-cache is intentional for this fixture
  return fetch('/data/user', { cache: 'force-cache' })
}
