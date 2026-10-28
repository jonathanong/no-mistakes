import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { page } from './helpers.mts'

// These imported calls have no inline executor callback and need no import lookup.
page('one')
page('two')
page('three')
page('four')
page('five')
page('six')
page('seven')
page('eight')
page('nine')
page('ten')
page('eleven')
page('twelve')

page(sql`/* bounded */ SELECT 1`, statement => read(statement))
page(sql`/* boundedAgain */ SELECT 2`, statement => read(statement))
