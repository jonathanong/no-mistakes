import { defineConfig } from '@playwright/test'
import shared from './playwright.root-spread-order-helper'

export default defineConfig({
  projects: [
    {
      name: 'root-spread-order-local',
      testMatch: ['local/**/*.spec.ts'],
    },
  ],
  ...shared,
})
