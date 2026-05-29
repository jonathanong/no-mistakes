import { makeConfig } from './vitest.root-call-cycle-a-helper'

export function makeConfigB() {
  return makeConfig()
}
