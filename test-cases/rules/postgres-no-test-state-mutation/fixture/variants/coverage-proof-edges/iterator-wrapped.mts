import { it } from 'vitest'

it.each((['wrapped'] as const) satisfies readonly string[])('accepted', field => { query(field) })
it.each((['non-null']!))('accepted', field => { query(field) })
