import { read } from '@example/db'

export function plain() {
  return read(`
SELECT id
FROM invoices
WHERE paid_at IS NULL
`)
}

export function escaped() {
  return read(`SELECT id\nFROM invoices\nWHERE paid_at IS NULL`)
}

const head = `
SELECT id
`
const tail = `
FROM invoices
WHERE paid_at IS NULL
`
export function concatenated() {
  return read(head + tail)
}
