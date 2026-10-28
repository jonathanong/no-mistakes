import { read } from '@bench/db'
import sql from 'sql-template-strings'

function named(flag: boolean) {
  if (flag) return sql`/* first */ SELECT 1`
  return sql`/* second */ SELECT 2`
}
function missing(flag: boolean) {
  if (flag) return sql`SELECT 1`
  return sql`SELECT 2`
}
function mixed(flag: boolean) {
  if (flag) return sql`/* mixed */ SELECT 1`
  return sql`SELECT 2`
}
function unknown(flag: boolean) {
  if (flag) return sql`/* fixed */ SELECT 1`
  return opaque()
}
function plain(flag: boolean) {
  if (flag) return '/* plainFirst */ SELECT 1'
  return '/* plainSecond */ SELECT 2'
}
function unreachable(_value: never): never { throw Error('invalid') }
function switched(value: 'a' | 'b') {
  switch (value) {
    case 'a': return sql`/* switchA */ SELECT 1`
    case 'b': return sql`/* switchB */ SELECT 2`
    default: return unreachable(value)
  }
}
function switchedMissing(value: 'a' | 'b') {
  switch (value) {
    case 'a': return sql`SELECT 1`
    case 'b': return sql`SELECT 2`
    default: throw Error('invalid')
  }
}
function noDefault(value: string) {
  switch (value) {
    case 'a': return sql`/* noDefaultA */ SELECT 1`
    case 'b': return sql`/* noDefaultB */ SELECT 2`
  }
}
function fallthrough(value: string) {
  switch (value) {
    case 'a':
    case 'b': return sql`/* fallthrough */ SELECT 1`
    default: throw Error('invalid')
  }
}
function alias(flag: boolean) {
  const value = opaque()
  if (flag) return sql`/* alias */ SELECT 1`
  return value
}
function builderAlias(flag: boolean) {
  const statement = sql`/* builderAlias */ SELECT 1`
  const alias = statement
  if (flag) return statement
  return alias
}
function mutableThrow(_value: never): never { throw Error('invalid') }
function mutableSwitch(value: 'a') {
  switch (value) {
    case 'a': return sql`/* mutable */ SELECT 1`
    default: return mutableThrow(value)
  }
}
mutableThrow = opaque
function shadowed(value: 'a') {
  function unreachable(_value: never): never { return opaque() }
  switch (value) {
    case 'a': return sql`/* shadowed */ SELECT 1`
    default: return unreachable(value)
  }
}
async function asynchronous(flag: boolean) {
  if (flag) return sql`/* asyncA */ SELECT 1`
  return sql`/* asyncB */ SELECT 2`
}
function* generator(flag: boolean) {
  if (flag) return sql`/* generatorA */ SELECT 1`
  return sql`/* generatorB */ SELECT 2`
}
function reassigned(flag: boolean) {
  if (flag) return sql`/* reassignedA */ SELECT 1`
  return sql`/* reassignedB */ SELECT 2`
}
reassigned = opaque

read(named(true)) // low: true
read(missing(true)) // low: false
read(mixed(true)) // low: false
read(unknown(true)) // low: false
read(plain(true)) // low: true
read(plain(true).append(tail)) // low: false
read(switched('a')) // low: true
read(switchedMissing('a')) // low: false
read(noDefault('a')) // low: false
read(fallthrough('a')) // low: false
read(alias(true)) // low: false
read(builderAlias(true)) // low: false
read(mutableSwitch('a')) // low: false
read(shadowed('a')) // low: false
read(asynchronous(true)) // low: false
read(generator(true)) // low: false
read(reassigned(true)) // low: false
read(named(true).append(tail)) // low: true
read(missing(true).append(tail)) // low: false
// Both executor reads keep the fresh builder local; the discarded append preserves its head.
const local = named(true)
read(local) // low: true
local.append(tail)
read(local) // low: true
