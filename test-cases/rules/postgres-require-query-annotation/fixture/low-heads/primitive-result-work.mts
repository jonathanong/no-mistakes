import { read } from '@bench/db'

function impossible(value) { throw new Error(String(value)) }
function named(type) {
  switch (type) {
    case 'one': return `/* primitive */ SELECT 1`
    case 'two': return `/* primitive */ SELECT 2`
    default: return impossible(type)
  }
}
function mixed(type) {
  switch (type) {
    case 'one': return `/* mixed */ SELECT 1`
    case 'two': return `SELECT 2`
    default: return impossible(type)
  }
}
function repeated(type) {
  const a = named(type); read(a) // low: true
  const b = named(type); read(b) // low: true
  const c = named(type); read(c) // low: true
  const d = named(type); read(d) // low: true
  const e = mixed(type); read(e) // low: false
  const f = mixed(type); read(f) // low: false
  const g = mixed(type); read(g) // low: false
  const h = mixed(type); read(h) // low: false
}
