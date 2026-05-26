export async function cachedFetches() {
  await fetch('/api/a', { cache: 'force-cache' })
  await fetch('/api/b', { next: { revalidate: false } })
  await fetch('/api/c', { next: { revalidate: 60 } })
  await fetch('/api/d', { next: { tags: ['user'] } })
}
