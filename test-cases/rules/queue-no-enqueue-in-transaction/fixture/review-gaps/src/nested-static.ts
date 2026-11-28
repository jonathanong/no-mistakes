import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

export class Outer {
  items: string[] = []

  run(job: { id: string }) {
    return withTx(async () => {
      class Inner {
        static items = { map: enqueue }
        static result = this.items.map(job)
      }
      return Inner.result
    })
  }
}
