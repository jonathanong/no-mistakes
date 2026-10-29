import { read } from '@bench/db'
import sql from 'sql-template-strings'

function propertyWrite() {
  const q = sql`/* propertyWrite */ SELECT 1`
  read(q) // low: false
  q.text = 'SELECT changed'
  read(q) // low: false
}

function letPropertyWriteAfterRead() {
  let q = sql`/* letPropertyWriteAfterRead */ SELECT 1`
  read(q) // low: false
  q.text = 'SELECT changed'
  read(q) // low: false
}

function assignWrite() {
  const q = sql`/* assignWrite */ SELECT 1`
  read(q) // low: false
  Object.assign(q, { text: 'SELECT changed' })
  read(q) // low: false
}

function reflectWrite() {
  const q = sql`/* reflectWrite */ SELECT 1`
  read(q) // low: false
  Reflect.set(q, 'text', 'SELECT changed')
  read(q) // low: false
}

function definePropertyWrite() {
  const q = sql`/* definePropertyWrite */ SELECT 1`
  read(q) // low: false
  Object.defineProperty(q, 'text', { value: 'SELECT changed' })
  read(q) // low: false
}

function deleteWrite() {
  const q = sql`/* deleteWrite */ SELECT 1`
  read(q) // low: false
  delete q.text
  read(q) // low: false
}

function stringsSpliceWrite() {
  const q = sql`/* stringsSpliceWrite */ SELECT 1`
  read(q) // low: false
  q.strings.splice(0, 1, 'SELECT changed')
  read(q) // low: false
}

function appendAfterRead() {
  const q = sql`/* appendAfterRead */ SELECT 1`
  read(q) // low: true
  q.append(' WHERE active')
  read(q) // low: true
}

function unknownPropertyReadAfterExecutor() {
  const q = sql`/* unknownPropertyReadAfterExecutor */ SELECT 1`
  read(q) // low: false
  void q.sql
}

function primitiveStringIsImmutable() {
  const q = '/* primitiveStringIsImmutable */ SELECT 1'
  read(q) // low: true
  Object.assign(q, { text: 'SELECT changed' })
  read(q) // low: true
}
