import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { first } from './first.mjs'

const query = first(sql`/* first */ SELECT `)
eval(source)
read(query.text, query.values)
