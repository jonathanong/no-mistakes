import { makeConfigB } from './vitest.root-call-cycle-b-helper'

export function makeConfig() {
  return makeConfigB()
}
