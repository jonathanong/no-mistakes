import { read } from '@bench/db'
import sql from 'sql-template-strings'
export function ordinary() { read(sql`/* ordinary */ SELECT 1`) }
