import sql from 'sql-template-strings'
import { read } from '@bench/db'

const query = sql`/* ordinaryRead */ SELECT 1`
read(query) // low: true
