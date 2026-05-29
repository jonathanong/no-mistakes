import { defineConfig } from '@playwright/test'

function shared() {
  return {
    projects: [
      {
        name: 'pw-root-call-spread',
        testMatch: ['pw-root-call-spread/**/*.spec.ts'],
      },
    ],
  }
}

const arrowShared = () => ({
  projects: [
    {
      name: 'pw-root-call-arrow-spread',
      testMatch: ['pw-root-call-arrow-spread/**/*.spec.ts'],
    },
  ],
})

const blockShared = () => {
  return {
    projects: [
      {
        name: 'pw-root-call-block-spread',
        testMatch: ['pw-root-call-block-spread/**/*.spec.ts'],
      },
    ],
  }
}

const functionShared = function () {
  return {
    projects: [
      {
        name: 'pw-root-call-function-spread',
        testMatch: ['pw-root-call-function-spread/**/*.spec.ts'],
      },
    ],
  }
}

const objectShared = {
  projects: [
    {
      name: 'pw-root-call-object-spread',
      testMatch: ['pw-root-call-object-spread/**/*.spec.ts'],
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
