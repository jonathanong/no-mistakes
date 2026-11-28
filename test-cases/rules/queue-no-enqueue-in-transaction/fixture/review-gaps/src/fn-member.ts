import { withTx } from './tx'
import { arrowHelper, helpers } from './fn-member-helper'

export function callFunctionMember(jobs: { id: string }[]) {
  return withTx(async () => {
    Array.from(jobs, helpers.send)
    Array.from(jobs, arrowHelper.send)
  })
}
