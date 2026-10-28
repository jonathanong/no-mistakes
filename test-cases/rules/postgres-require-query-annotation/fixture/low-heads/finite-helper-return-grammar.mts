import { read } from '@bench/db'
import sql from 'sql-template-strings'
import otherSql from '@other/sql'

function alternate(flag: boolean) {
  if (flag) return sql`/* alternate */ SELECT 1`
  else return sql`/* alternate */ SELECT 2`
  return sql`/* alternate */ SELECT 3`
}
function alternateBlock(flag: boolean) {
  if (flag) { return sql`/* alternateBlock */ SELECT 1` }
  else { return sql`/* alternateBlock */ SELECT 2` }
  return sql`/* alternateBlock */ SELECT 3`
}
function defaultBreak(value: string) {
  switch (value) {
    case 'a': return sql`/* defaultBreak */ SELECT 1`
    default: break
  }
}
function nondefaultThrow(value: string) {
  switch (value) {
    case 'a': throw Error('unsupported')
    case 'b': return sql`/* nondefaultThrow */ SELECT 1`
    default: throw Error('unsupported')
  }
}
function nondefaultBreak(value: string) {
  switch (value) {
    case 'a': break
    case 'b': return sql`/* nondefaultBreak */ SELECT 1`
    default: throw Error('unsupported')
  }
}
function blockReturn(flag: boolean) {
  if (flag) { return sql`/* blockReturn */ SELECT 1` }
  return sql`/* blockReturn */ SELECT 2`
}
function untrustedTag(flag: boolean) {
  if (flag) return otherSql`/* untrustedTag */ SELECT 1`
  return otherSql`/* untrustedTag */ SELECT 2`
}
function appendPrefix(flag: boolean) {
  if (flag) return sql`/* appendPrefix */ SELECT 1`.append(tail)
  return sql`/* appendPrefix */ SELECT 2`.append(tail)
}
function textPrefix(flag: boolean) {
  if (flag) return `/* textPrefix */ SELECT ${tail}`
  return `/* textPrefix */ SELECT ${tail}`
}
function onlyThrows(): never {
  throw Error('never reaches SQL')
}
function opaqueMissing(flag: boolean) {
  if (flag) return sql`SELECT 1`.append(tail)
  return sql`SELECT 2`.append(tail)
}

read(alternate(true)) // low: true
read(alternateBlock(true)) // low: true
read(defaultBreak('a')) // low: false
read(nondefaultThrow('b')) // low: true
read(nondefaultBreak('b')) // low: false
read(blockReturn(true)) // low: true
read(untrustedTag(true)) // low: false
read(appendPrefix(true)) // low: true
read(textPrefix(true)) // low: true
read(onlyThrows()) // low: false
read(opaqueMissing(true)) // low: false
