const mode = 'force-cache'
const alias = mode
const seconds = 60
const policy = 'force-static'
const fetchMode = 'only-cache'
const off = false

export const revalidate = seconds
export const dynamic = policy
export const fetchCache = fetchMode

export default async function Page() {
  const loadLater = () => fetch('/api/later', { cache: laterMode })
  const laterMode = 'force-cache'
  await loadLater()
  const localMode = 'force-cache'
  await fetch('/api/template', { cache: `force-cache` })
  await fetch('/api/key', { [`cache`]: alias })
  await fetch('/api/local', { cache: localMode })
  await fetch('/api/seconds', { next: { revalidate: seconds } })
  await fetch('/api/off', { next: { revalidate: off } })
  if (process.env.READY) {
    const blocked = 'force-cache'
    await fetch('/api/blocked', { cache: blocked })
  }
  return null
}
