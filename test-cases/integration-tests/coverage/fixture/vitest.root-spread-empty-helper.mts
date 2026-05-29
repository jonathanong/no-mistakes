const specifierConfig = {
  projects: [
    {
      test: {
        name: 'ignored-specifier-config',
        include: ['ignored-specifier-config/**/*.test.ts'],
      },
    },
  ],
}

const source = {}

export { specifierConfig }
export { sourcedConfig } from './vitest.root-spread-sourced'
export { reexportedSourcedConfig } from './vitest.root-spread-sourced'
export const { destructuredConfig } = source
export const unrelatedConfig = {}
