import * as cyclic from './namespace-cycle-b.ts'
import { cycleValue } from './namespace-cycle-b.ts'
declare function consume(value: unknown): void
consume(cyclic)
const CATALOG = { github: { emailColumn: 'github_email' } } as const
export function getCycle(provider: keyof typeof CATALOG) { return CATALOG[provider] }
const config = getCycle('github')
void config.emailColumn
export const cycleMarker = cycleValue
