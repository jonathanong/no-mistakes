import { read } from '@bench/db'
import sql from 'sql-template-strings'
export function ordinary() { const q = sql`/* named */ SELECT 1`; read(q); }
