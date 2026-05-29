import { defineConfig } from '@playwright/test'

// testIgnore with an invalid value exercises the error path of '?' in merge_property (objects.rs:102)
export default defineConfig({
  projects: [
    {
      testDir: './playwright-testignore-invalid',
      // @ts-ignore
      testIgnore: notAStringOrArray,
    },
  ],
})
