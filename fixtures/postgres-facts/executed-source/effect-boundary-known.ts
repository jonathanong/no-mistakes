import sql from 'sql-template-strings'
import { read } from '@example/db'
import { appendLiteral } from './effect-boundary-helper'

export function readAfterKnownAppend() {
  const query = sql`/* effects/known */ SELECT id FROM posts`
  appendLiteral(query)
  return read(query)
}
