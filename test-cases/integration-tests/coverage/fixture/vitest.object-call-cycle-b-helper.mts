import { makeProject } from './vitest.object-call-cycle-a-helper'

export function makeProjectB() {
  return makeProject()
}
