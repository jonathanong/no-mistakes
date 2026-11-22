import { configDefaults } from 'vitest/config'

export default {
  test: {
    exclude: [],
    projects: [
      { test: { exclude: ['tmp/**'] } },
      { test: { exclude: [...configDefaults.exclude] } },
    ],
  },
}
