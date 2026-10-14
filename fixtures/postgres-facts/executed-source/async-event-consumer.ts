import sql from 'sql-template-strings'
import { read } from '@example/db'

async function readAfterGate(query: typeof sql) {
  await Promise.resolve()
  return read(query)
}

export function startThenMutate() {
  const query = sql`/* async/event */ SELECT id FROM posts`
  const pending = readAfterGate(query)
  query.append(' UPDATE posts SET title = title')
  void pending
}
