import { defineConfig } from '@playwright/test'
import { missingBases } from './playwright.object-member-defensive-missing'
import { arrayBases, emptyBases } from './playwright.object-member-defensive-source'

const localEmptyBases = {}
const localArrayBases = {
  web: ['not an object'],
}

export default defineConfig({
  projects: [
    { ...localEmptyBases.web, name: 'pw-local-empty-member' },
    { ...localArrayBases.web, name: 'pw-local-array-member' },
    { ...missingBases.web, name: 'pw-missing-import-member' },
    { ...emptyBases.web, name: 'pw-empty-import-member' },
    { ...arrayBases.web, name: 'pw-array-import-member' },
  ],
})
