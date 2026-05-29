import { defineConfig } from 'vitest/config'

// Self-recursive function - covers local_seen cycle detection (line 23)
function makeConfig(): any {
  return { ...makeConfig() }
}

export default defineConfig({
  ...makeConfig(),
})
