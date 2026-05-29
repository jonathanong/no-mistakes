import { makeConfigB } from './playwright.root-call-cycle-b-helper'

export function makeConfig() {
  return makeConfigB()
}
