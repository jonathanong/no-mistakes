import { defineConfig } from 'vitest/config'
import { starBarrelConfig } from './vitest.root-spread-star-barrel'

// starBarrelConfig is found via export* chain in the barrel
export default defineConfig({
  ...starBarrelConfig,
})
