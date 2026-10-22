let mode = 'force-cache'
var policy = 'force-static'
const key = 'cache'
const kind = 'cache'
const zero = 0

export const revalidate = zero
export const dynamic = 'force-dynamic'
export const fetchCache = 'default-no-store'

export default async function Page() {
  await fetch('/api/let', { cache: mode })
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
