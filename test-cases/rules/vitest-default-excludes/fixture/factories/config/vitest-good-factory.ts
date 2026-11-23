import { configDefaults } from 'vitest/config';

export function createDefaultSpreadConfig() {
  return {
    test: {
      exclude: [...configDefaults.exclude, 'custom/**'],
    },
  };
}
