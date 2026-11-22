import { configDefaults as defaults } from 'vitest/config'

export default {
  test: {
    exclude: ['**/node_modules/**', '**/.git/**'],
    projects: [
      { test: { exclude: ['**/dist/**'] } },
      { test: { exclude: [...defaults.exclude] } },
    ],
  },
}
