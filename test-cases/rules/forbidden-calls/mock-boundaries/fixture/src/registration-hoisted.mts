import { vi } from 'vitest'
import { forbidden } from './forbidden.mts'
// Vitest hoists mock registration even from an uncalled helper.
function dormant() {
  vi.mock('./analysis.mts', () => { forbidden(); return { analyze: () => null } })
}
export function entry() {}
