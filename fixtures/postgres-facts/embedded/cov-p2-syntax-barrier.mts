import { read } from '@example/db'
declare function opaque(value: unknown): void
export function destructuringEscape() {
  let target
  const box = Object.freeze({ query: read }) // UNKNOWN:read
  ;({ target } = { target: box })
  opaque(target)
}
read('SELECT 1') // SQL:read('SELECT 1')=>SELECT 1
