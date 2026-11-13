import { read, type QueryExecutor } from '@example/db'
declare function opaque(value: unknown): void
declare const dynamicSql: string
export function ownData() {
  const box = Object.freeze({ query: read, metadata: { hostname: 'local' }, run() { this.metadata.hostname } })
  box.run() // QUIET
  read('SELECT 1') // SQL:read('SELECT 1')=>SELECT 1
}
export function unknownMethod(run: () => void) {
  const box = Object.freeze({ query: read, run })
  box.run() // UNKNOWN:box
}
export function receiverEscapes() {
  const box = Object.freeze({ query: read, run() { opaque(this) } })
  box.run() // UNKNOWN:box
}
export function computedReceiver(key: string) {
  const box = Object.freeze({ query: read, run() { opaque(this[key]) } })
  box.run() // UNKNOWN:box
}
export function overlapping(query: QueryExecutor) {
  const box = { query, run() { opaque(this) } }
  ;(box.query || box.run)(dynamicSql) // UNKNOWN:(box.query || box.run)
  box.query.call(box, dynamicSql) // UNKNOWN:box.query.call
}
