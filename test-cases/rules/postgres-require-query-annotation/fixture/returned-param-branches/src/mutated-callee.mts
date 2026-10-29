import { read } from '@bench/db'
import sql from 'sql-template-strings'
import custom from '@bench/custom'
import { factory, unproven } from './factories.mjs'
declare const flag: boolean, extra: any, other: any
declare function inspect(value: unknown): any
factory = other
export function rejected() { const q = factory(sql`/* changed */ SELECT `, extra); read(q.text, q.values) }
