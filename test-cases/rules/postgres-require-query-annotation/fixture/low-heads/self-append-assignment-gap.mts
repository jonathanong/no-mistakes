import { read } from '@bench/db'
import sql from 'sql-template-strings'
import custom from '@bench/custom'
declare const n: number, flag: boolean, other: any
declare function outside(value: unknown): unknown
export function gap() { let q = sql`/* named */ SELECT ${n}`; q = q.append(sql` suffix`); eval("unknown"); read(q); }
