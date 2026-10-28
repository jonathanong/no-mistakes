import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { page } from './helpers.mts'
declare const source: any

page(sql`/* control */ SELECT 1`, statement => read(statement))
// The helper does not receive an exact inline executor from these callbacks.
page(sql`/* destructured */ SELECT 1`, ({ statement }) => read(statement))
page(sql`/* body */ SELECT 1`, statement => { return read(statement) })
page(sql`/* optional */ SELECT 1`, statement => read?.(statement))
page(sql`/* arity */ SELECT 1`, statement => read(statement, {}))
// A member tag is an opaque payload even when the callback itself is exact.
page(source.sql`/* untrusted */ SELECT 1`, statement => read(statement))
