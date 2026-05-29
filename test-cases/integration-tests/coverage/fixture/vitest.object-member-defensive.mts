import { defineConfig } from 'vitest/config'
import { missingBases } from './vitest.object-member-defensive-missing'
import { arrayBases, emptyBases } from './vitest.object-member-defensive-source'

const localEmptyBases = {}
const localArrayBases = {
  web: ['not an object'],
}

export default defineConfig({
  test: {
    projects: [
      { ...localEmptyBases.web, test: { name: 'vitest-local-empty-member' } },
      { ...localArrayBases.web, test: { name: 'vitest-local-array-member' } },
      { ...missingBases.web, test: { name: 'vitest-missing-import-member' } },
      { ...emptyBases.web, test: { name: 'vitest-empty-import-member' } },
      { ...arrayBases.web, test: { name: 'vitest-array-import-member' } },
    ],
  },
})
