import { defineConfig } from 'vitest/config'
import * as allBases from './vitest.member-spread-namespace-source'

const merged = { ...allBases }

export default defineConfig({
  test: {
    projects: merged.web,
  },
})
