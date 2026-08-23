import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    name: 'absolute-setup',
    setupFiles: ['__ABSOLUTE_RUNTIME_SETUP__', '__ABSOLUTE_DECLARATION_SETUP__'],
  },
})
