import { cycleConfigFromB } from './vitest.root-spread-cycle-b'

export const cycleConfig = {
  test: {
    ...cycleConfigFromB,
  },
}
