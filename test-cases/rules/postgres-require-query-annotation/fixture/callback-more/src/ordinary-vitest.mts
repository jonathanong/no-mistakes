import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { it } from 'vitest'

// The read is ordinary local-const SQL; this zero-parameter test callback does
// not forward a producer payload and must not construct callback proof indexes.
it('ordinary read inside a test callback', async () => {
  const query = sql`/* ordinaryVitestRead */ SELECT 1`
  await read(query)
})
