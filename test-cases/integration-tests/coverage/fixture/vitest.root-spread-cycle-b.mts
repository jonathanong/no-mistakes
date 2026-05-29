import { cycleConfig } from './vitest.root-spread-cycle-a'

export const cycleConfigFromB = {
  test: {
    ...cycleConfig,
  },
}
