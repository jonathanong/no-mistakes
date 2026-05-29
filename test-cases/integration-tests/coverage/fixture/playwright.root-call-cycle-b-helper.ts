import { makeConfig } from './playwright.root-call-cycle-a-helper'

export function makeConfigB() {
  return makeConfig()
}
