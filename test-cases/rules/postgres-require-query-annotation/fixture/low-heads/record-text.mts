import { read } from '@bench/db'

function localText(value: unknown) {
  const text = `/* localText */ SELECT ${value}`
  return { text, values: [value] }
}
function localMissing() {
  const text = 'SELECT 1'
  return { text, values: [] }
}
function parameterText(text: string) {
  return { text, values: [] }
}
function changedText() {
  let text = '/* changed */ SELECT 1'
  text = 'SELECT 1'
  return { text, values: [] }
}
function destructured(value: unknown) {
  return { text: `/* destructured */ SELECT ${value}`, values: [value] }
}
function destructuredMissing() {
  return { text: 'SELECT 1', values: [] }
}
function builder(value: unknown) {
  return sql`/* builder */ SELECT ${value}`
}
function getter(value: unknown) {
  return { get text() { return '/* getter */ SELECT 1' }, values: [value] }
}

function use(value: unknown) {
  const local = localText(value)
  read(local.text, local.values) // low: true
  const missing = localMissing()
  read(missing.text, missing.values) // low: false
  const parameter = parameterText('/* fake */ SELECT 1')
  read(parameter.text, parameter.values) // low: false
  const changed = changedText()
  read(changed.text, changed.values) // low: false
  for (const item of [value]) {
    const { text, values } = destructured(item)
    read(text, values) // low: true
  }
  {
    const { text, values } = destructuredMissing()
    read(text, values) // low: false
  }
  // Exact shorthand bindings are required for the renamed case.
  const { text: absentText, values: absentValues } = destructuredMissing()
  read(absentText, absentValues) // low: false
  const { text: wrongText, values: wrongValues } = builder(value)
  read(wrongText, wrongValues) // low: false
  const { text: getterText, values: getterValues } = getter(value)
  read(getterText, getterValues) // low: false
}
