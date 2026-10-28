import { read } from '@bench/db'
import sql from 'sql-template-strings'
declare const n: number, other: any
sql = other
export function invalidImport() { let q = sql`/* named */ SELECT ${n}`; q = q.append(sql` suffix`); read(q); }
