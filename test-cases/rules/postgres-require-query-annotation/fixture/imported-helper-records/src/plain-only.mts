// Plain result proofs must not borrow the trusted builder's output kind.
import sql from 'sql-template-strings'
export function pureRecord(value: unknown) {
  return { text: `/* pureImportedRecord */ SELECT ${value}`, values: [value] }
}

export function localText(value: unknown) {
  const text = `/* importedLocalText */ SELECT ${value}`
  return { text, values: [value] }
}

export function destructured(value: unknown) {
  return { text: `/* importedDestructure */ SELECT ${value}`, values: [value] }
}

export function builderOnly(value: unknown) {
  return sql`/* builderOnly */ SELECT ${value}`
}
