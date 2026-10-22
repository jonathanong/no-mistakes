const mode = 'force-cache'
var policy = 'force-static'
const key = 'cache'
const kind = 'cache'
const zero = 0

export const revalidate = zero
export const dynamic = 'force-dynamic'
export const fetchCache = 'default-no-store'

export default async function Page(mode: string) {
  await fetch('/api/param', { cache: mode })
  let local = 'force-cache'
  await fetch('/api/let-shadow', { cache: local })
  await fetch('/api/key', { [key]: 'force-cache' })
  await fetch('/api/template', { cache: `force-${kind}` })
  await fetch('/api/zero', { next: { revalidate: zero } })
  let off = false
  await fetch('/api/off', { next: { revalidate: off } })
  {
    const hidden = 'force-cache'
  }
  await fetch('/api/hidden', { cache: hidden })
  return policy
}

async function extra(mode = 'force-cache', ...rest: string[]) {
  const { cache: local, ...ignored } = { cache: 'force-cache' }
  const [first, ...more] = ['force-cache']
  const peek = (value: string) => fetch('/api/peek', { cache: value })
  await peek(mode)
  await fetch('/api/bound', { cache: local })
  await fetch('/api/first', { cache: first })
  await fetch('/api/computed-key', { [`cache-${kind}`]: 'force-cache' })
  return rest.length + more.length + ignored.cache + first
}
