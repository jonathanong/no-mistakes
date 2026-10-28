import { read } from '@bench/db'
import sql from 'sql-template-strings'

function namedPlain(value: unknown) {
  return { text: `/* namedPlainRecord */ SELECT ${value}`, values: [value] }
}
function missingPlain() {
  return { text: 'SELECT 1', values: [] }
}
function namedBuilder(value: unknown) {
  return sql`/* namedBuilderRecord */ SELECT ${value}`
}
function missingBuilder() {
  return sql`SELECT 1`
}
function parameterText(text: string) {
  return { text, values: [] }
}
function getterText() {
  return { get text() { return '/* getter */ SELECT 1' }, values: [] }
}
function spreadText() {
  return { ...other, text: '/* spread */ SELECT 1', values: [] }
}
function duplicateText() {
  return { text: '/* first */ SELECT 1', text: 'SELECT 1', values: [] }
}
function otherPath(flag: boolean) {
  if (flag) return { text: 'SELECT 1', values: [] }
  return { text: '/* otherPath */ SELECT 1', values: [] }
}
let reassigned = namedPlain
reassigned = parameterText

function usage(value: unknown) {
  const plain = namedPlain(value)
  read(plain.text, plain.values) // low: true
  const absent = missingPlain()
  read(absent.text, absent.values) // low: false
  const builder = namedBuilder(value)
  read(builder.text, builder.values) // low: true
  const unmarked = missingBuilder()
  read(unmarked.text, unmarked.values) // low: false
  const parameter = parameterText('/* forged */ SELECT 1')
  read(parameter.text, parameter.values) // low: false
  const getter = getterText()
  read(getter.text, getter.values) // low: false
  const spread = spreadText()
  read(spread.text, spread.values) // low: false
  const duplicate = duplicateText()
  read(duplicate.text, duplicate.values) // low: false
  const branch = otherPath(true)
  read(branch.text, branch.values) // low: false
  const changed = namedPlain(value)
  changed.text = 'SELECT 1'
  read(changed.text, changed.values) // low: false
  const escaped = namedPlain(value)
  unknown(escaped)
  read(escaped.text, escaped.values) // low: false
  const alias = namedPlain(value)
  const copy = alias
  read(alias.text, alias.values) // low: false
  const reassignedResult = reassigned(value)
  read(reassignedResult.text, reassignedResult.values) // low: false
  const optional = namedPlain(value)
  read(optional?.text, optional.values) // low: false
  const computed = namedPlain(value)
  read(computed['text'], computed.values) // low: false
  const optionalCall = namedPlain(value)
  read?.(optionalCall.text, optionalCall.values) // low: false
}

function customBuilder() {
  return customTag`/* customRecord */ SELECT 1`
}
function rejectedCustomAndOptions() {
  const custom = customBuilder()
  read(custom.text, custom.values) // low: false
  const options = namedPlain(1)
  read(options.text, options.values, {}) // low: false
}
function rejectedRepeated() {
  const record = namedPlain(1)
  for (const item of items) {
    read(record.text, record.values) // low: false
  }
}
