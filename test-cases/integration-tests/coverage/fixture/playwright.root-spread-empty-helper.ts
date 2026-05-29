const specifierConfig = {
  projects: [
    {
      name: 'ignored-specifier-config',
      testMatch: ['ignored-specifier-config/**/*.spec.ts'],
    },
  ],
}

const source = {}

export { specifierConfig }
export { sourcedConfig } from './playwright.root-spread-sourced'
export { reexportedSourcedConfig } from './playwright.root-spread-sourced'
export const { destructuredConfig } = source
export const unrelatedConfig = {}
