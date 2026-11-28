import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

export function nested() {
  return withTx(async (query) => {
    return query.items.map(() => enqueue({ id: '1' }))
  })
}
