import { read } from '@bench/db'
import sql from 'sql-template-strings'
declare function consume(value: unknown): void

function named() {
  return { text: '/* namedRecord */ SELECT 1', values: [] }
}
function extraOwnField() {
  return { text: '/* extraOwnField */ SELECT 1', values: [], metadata: 1 }
}
function computedText() {
  return { text: getSql(), values: [] }
}
function prototypeField() {
  return { text: '/* prototypeField */ SELECT 1', values: [], __proto__: other }
}
function builder() {
  return sql`/* builderKind */ SELECT 1`
}

function use() {
  const { text: [nestedText], values: nestedValues } = named()
  read(nestedText, nestedValues) // low: false
  {
    const { text, other } = named()
    read(text, other) // low: false
  }
  {
    const { text = 'SELECT 0', values } = named()
    read(text, values) // low: false
  }
  const { text: firstText, text: secondText } = named()
  read(firstText, secondText) // low: false
  const { text, values } = named()
  read(text, ...[values]) // low: false
  const extra = extraOwnField()
  read(extra.text, extra.values) // low: true
  const computed = computedText()
  read(computed.text, computed.values) // low: false
  const prototype = prototypeField()
  read(prototype.text, prototype.values) // low: false
  {
    // A trusted builder is still a different output kind than a plain record.
    const { text, values } = builder()
    read(text, values) // low: false
  }
  {
    const { text, values } = named()
    consume(text)
    read(text, values) // low: false
  }
}
