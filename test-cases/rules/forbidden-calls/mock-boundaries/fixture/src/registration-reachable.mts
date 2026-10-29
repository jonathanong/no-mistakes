import { vi } from 'vitest'
import { forbidden } from './forbidden.mts'
function register() {
  vi.doMock('./analysis.mts', () => { forbidden(); return { analyze: () => null } })
}
export function entry() { register() }
entry()
