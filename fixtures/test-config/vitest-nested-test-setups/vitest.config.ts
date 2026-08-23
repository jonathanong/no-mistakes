import { nestedImported, outerImported } from './imported-spreads'

const nestedLocal = {
  test: {
    name: 'local-nested-spread',
    include: ['local-nested-spread/**/*.test.ts'],
    setupFiles: './inner-local.ts',
    globalSetup: './inner-local-global.ts',
  },
}

const outerLocal = {
  setupFiles: './outer-local.ts',
  globalSetup: './outer-local-global.ts',
}

export default {
  test: {
    projects: [
      {
        // Outer setup fields precede `test`, but Vitest still ignores them.
        setupFiles: './outer-first.ts',
        globalSetup: './outer-first-global.ts',
        test: {
          name: 'outer-first',
          include: ['outer-first/**/*.test.ts'],
          setupFiles: './inner-first.ts',
          globalSetup: './inner-first-global.ts',
        },
      },
      {
        test: {
          name: 'outer-last',
          include: ['outer-last/**/*.test.ts'],
          setupFiles: './inner-last.ts',
          globalSetup: './inner-last-global.ts',
        },
        // Property order cannot make these outer setup fields effective.
        setupFiles: './outer-last.ts',
        globalSetup: './outer-last-global.ts',
      },
      {
        // A local outer spread after nested test fields remains ineffective.
        ...nestedLocal,
        ...outerLocal,
      },
      {
        // The local outer spread is also ineffective before nested test fields.
        ...outerLocal,
        ...nestedLocal,
      },
      {
        // Imported static spreads preserve nested-test setup precedence too.
        ...nestedImported,
        ...outerImported,
      },
      {
        ...outerImported,
        ...nestedImported,
      },
    ],
  },
}
