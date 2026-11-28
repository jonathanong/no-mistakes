import { withTx } from './tx'
import { enqueue } from '@sample/jobs'
import { jobs } from './cycle-a'

export function viaCycle() {
  return withTx(async () => {
    jobs.enqueue({ id: '1' })
    return enqueue({ id: '2' })
  })
}
