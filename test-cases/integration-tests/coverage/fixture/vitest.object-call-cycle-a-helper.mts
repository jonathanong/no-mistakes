import { makeProjectB } from './vitest.object-call-cycle-b-helper'

export function makeProject() {
  return makeProjectB()
}
