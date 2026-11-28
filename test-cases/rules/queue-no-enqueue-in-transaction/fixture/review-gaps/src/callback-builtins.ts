import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

export function publish(): Promise<unknown> {
  return withTx(async () => {
    const viaTry = Array.from([enqueue], Promise.try, Promise)
    return [1, -2].map(Math.abs).concat(viaTry)
  })
}
