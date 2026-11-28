import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

export function parse(payload: string) {
  return withTx(async () => {
    const skipped = JSON.parse(payload, 'reviver')
    const revived = JSON.parse(payload, () => enqueue({ id: '1' }))
    return { skipped, revived }
  })
}
