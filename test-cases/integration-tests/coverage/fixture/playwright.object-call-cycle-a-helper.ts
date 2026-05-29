import { makeProjectB } from './playwright.object-call-cycle-b-helper'

export function makeProject() {
  return makeProjectB()
}
