import sql from 'sql-template-strings'
import { read } from '@example/db'
import { appendAfterGate } from './effect-boundary-helper'

export async function readAfterSuspendedAppend() {
  const query = sql`/* effects/suspended */ SELECT id FROM posts`
  await appendAfterGate(query)
  return read(query)
}
