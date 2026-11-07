import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { first } from './first.mjs'
import { second } from './second.mjs'
import { absent } from './unresolved.mjs'

const distinct = flag ? first(sql`/* first */ SELECT `) : second(sql`/* second */ SELECT `)
read(distinct.text, distinct.values)

const nested = flag ? first(sql`/* first */ SELECT `) : other ? second(sql`/* second */ SELECT `) : first(sql`/* third */ SELECT `)
nested.append(' LIMIT 1')
read(nested.text, nested.values)

const unresolved = flag ? first(sql`/* first */ SELECT `) : absent(sql`/* absent */ SELECT `)
read(unresolved.text, unresolved.values)

const missingValues = first(sql`/* first */ SELECT `)
read(missingValues.text, [])

const singleField = first(sql`/* first */ SELECT `)
read(singleField.text)

const appendAfter = first(sql`/* first */ SELECT `)
read(appendAfter.text, appendAfter.values)
appendAfter.append(' LIMIT 1')

const selfReference = selfReference ? first(sql`/* first */ SELECT `) : second(sql`/* second */ SELECT `)
read(selfReference.text, selfReference.values)

const depth8 = flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : first(sql`/* bounded */ SELECT `)
read(depth8.text, depth8.values)

const depth9 = flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : flag ? first(sql`/* bounded */ SELECT `) : first(sql`/* bounded */ SELECT `)
read(depth9.text, depth9.values)
