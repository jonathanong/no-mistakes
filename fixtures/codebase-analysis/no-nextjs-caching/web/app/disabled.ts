// guardrails-disable-file nextjs-no-caching
import { unstable_cache } from 'next/cache'

export const revalidate = 60
export const ignored = unstable_cache(async () => 1)
