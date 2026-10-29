import { vi } from 'vitest'
import { forbidden } from './forbidden.mts'
// Module and function roots must not execute this uncalled registration.
function dormant() {
  vi.doMock('./analysis.mts', () => { forbidden(); return { analyze: () => null } })
}
export function entry() {}
