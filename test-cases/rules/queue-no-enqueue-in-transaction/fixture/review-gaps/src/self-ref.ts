import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

const ready = false
const items: number[] = ready ? [] : items.map((item) => item)

export function walkItems() {
  return withTx(async () => {
    return items.map(() => enqueue({ id: '1' }))
  })
}
