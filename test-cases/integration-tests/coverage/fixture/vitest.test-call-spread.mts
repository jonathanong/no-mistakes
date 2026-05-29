import { defineConfig } from 'vitest/config'

function shared() {
  return {
    projects: [
      {
        test: {
          name: 'vitest-test-call-spread',
          include: ['vitest-test-call-spread/**/*.test.ts'],
        },
      },
    ],
  }
}

const arrowShared = () => ({
  projects: [
    {
      test: {
        name: 'vitest-test-call-arrow-spread',
        include: ['vitest-test-call-arrow-spread/**/*.test.ts'],
      },
    },
  ],
})

const blockShared = () => {
  return {
    projects: [
      {
        test: {
          name: 'vitest-test-call-block-spread',
          include: ['vitest-test-call-block-spread/**/*.test.ts'],
        },
      },
    ],
  }
}

const functionShared = function () {
  return {
    projects: [
      {
        test: {
          name: 'vitest-test-call-function-spread',
          include: ['vitest-test-call-function-spread/**/*.test.ts'],
        },
      },
    ],
  }
}

const objectShared = {
  projects: [
    {
      test: {
        name: 'vitest-test-call-object-spread',
        include: ['vitest-test-call-object-spread/**/*.test.ts'],
      },
    },
  ],
}

const recursiveShared = () => recursiveShared()

function noReturnShared() {
  const ignored = true
}

function returnOnlyShared() {
  return
}

export default defineConfig({
  test: {
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
  },
})
