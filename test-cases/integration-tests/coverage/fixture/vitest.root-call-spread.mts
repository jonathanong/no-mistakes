import { defineConfig } from 'vitest/config'

function shared() {
  return {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-call-spread',
            include: ['vitest-root-call-spread/**/*.test.ts'],
          },
        },
      ],
    },
  }
}

const arrowShared = () => ({
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-call-arrow-spread',
          include: ['vitest-root-call-arrow-spread/**/*.test.ts'],
        },
      },
    ],
  },
})

const blockShared = () => {
  return {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-call-block-spread',
            include: ['vitest-root-call-block-spread/**/*.test.ts'],
          },
        },
      ],
    },
  }
}

const functionShared = function () {
  return {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-call-function-spread',
            include: ['vitest-root-call-function-spread/**/*.test.ts'],
          },
        },
      ],
    },
  }
}

const objectShared = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-call-object-spread',
          include: ['vitest-root-call-object-spread/**/*.test.ts'],
        },
      },
    ],
  },
}

const recursiveShared = () => recursiveShared()

function noReturnShared() {
  const ignored = true
}

function returnOnlyShared() {
  return
}

export default defineConfig({
  ...({}).shared(),
  ...missingShared(),
  ...recursiveShared(),
  ...noReturnShared(),
  ...returnOnlyShared(),
  ...objectShared(),
  ...arrowShared(),
  ...blockShared(),
  ...functionShared(),
  ...shared(),
})
