import sql from 'sql-template-strings'
import { read } from '@example/db'
import { appendLiteral } from './effect-boundary-helper'

export function readAfterIndirectAppend() {
  const query = sql`/* effects/indirect */ SELECT id FROM posts`
  appendLiteral([query][0])
  return read(query)
}
