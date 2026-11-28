import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

const values = new Map<string, unknown>()
values.set = enqueue as never
let count = 0
count = 1

export function reassignedMethod(job: { id: string }) {
  return withTx(async () => {
    values.set('job', job)
    const keep = values.get('job')
    return { keep, count }
  })
}
