import { withTx } from './tx'
import * as jobs from './self-ns'

export function nestedNamespace() {
  return withTx(async () => {
    Array.from([{ id: '1' }], jobs.nested.nested.enqueue)
  })
}
