import type { Status } from '@example/lifecycle-types'
export function helper(status: Status, at: Date) {
  return {
    aAt: status === 'x' ? at : null,
    bAt: status === 'y' ? at : null,
    cAt: status === 'z' ? at : null,
    dAt: status === 'paused' ? at : null,
  }
}
export async function recordChange(id: string) {
  return id
}
