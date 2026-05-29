import { defineConfig } from '@playwright/test'

// Self-recursive function - covers local_seen cycle detection (line 22)
function makeConfig(): any {
  return { ...makeConfig() }
}

export default defineConfig({
  ...makeConfig(),
})
