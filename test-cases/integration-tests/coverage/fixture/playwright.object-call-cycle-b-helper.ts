import { makeProject } from './playwright.object-call-cycle-a-helper'

export function makeProjectB() {
  return makeProject()
}
