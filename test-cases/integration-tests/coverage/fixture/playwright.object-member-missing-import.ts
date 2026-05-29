import { defineConfig } from '@playwright/test'
import { bases } from '@missing-no-mistakes-pkg'

export default defineConfig({
  projects: [
    {
      ...bases.web,
      name: 'pw-object-member-missing-import',
    },
  ],
})
