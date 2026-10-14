import { read } from '@example/db'
import { afterComputedGate } from './effect-boundary-nested-await'

export async function readAfterComputedGate() {
  return read(await afterComputedGate())
}
