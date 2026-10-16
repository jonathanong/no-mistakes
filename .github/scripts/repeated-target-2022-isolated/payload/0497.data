import sql from 'sql-template-strings'
import { read } from '@example/db'

export function readWithGetter() {
  const query = sql`/* executor getter */ SELECT 1`
  const options = {
    get trigger() {
      query.append(' UPDATE posts SET title = title')
      return 1
    },
  }
  return read(query, options.trigger)
}
