import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { selectWithOwner } from './other.mjs'

const query = selectWithOwner(sql`/* uniqueImport */ SELECT `)
read(query.text, query.values) // low: false
