import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

export class Runner {
  queue: { push(job: string): void } = { push() {} }
  items: string[] = []

  run(): Promise<string[]> {
    return withTx(async () => {
      this.queue.push('job')
      const own = this.items.map((item) => item.trim())
      await enqueue({ id: '1' })
      return own
    })
  }
}
