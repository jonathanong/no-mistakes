import sql from 'sql-template-strings'
import { read } from '@example/db'

const api = {
  query: sql`/* receiver argument */ SELECT 1`,
  execute(_value: number) {
    return read(this.query)
  },
}

function append(query: typeof sql) {
  query.append(' UPDATE posts SET title = title')
  return 1
}

export function run() {
  return api.execute(append(api.query))
}
