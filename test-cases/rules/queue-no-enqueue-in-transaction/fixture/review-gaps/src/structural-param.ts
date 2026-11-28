import { withTx } from './tx'

export function send(queue: { push(job: { id: string }): void }, job: { id: string }) {
  return withTx(async () => {
    queue.push(job)
  })
}
