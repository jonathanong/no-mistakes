import { read } from '@example/db'
import sql from 'sql-template-strings'

const query = sql`/* getter/read */ SELECT id FROM posts`
const object = {
  get trigger() {
    query.append(sql` UPDATE posts SET title = 'changed'`)
    return true
  },
}

export function list() {
  const ignored = object.trigger
  void ignored
  return read(query)
}
