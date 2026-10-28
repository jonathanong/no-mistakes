import sql from 'sql-template-strings'
import { read } from '@bench/db'

function impossible(value) { throw new Error(String(value)) }
function named(type) {
  switch (type) {
    case 'one': return `/* readOne */ SELECT 1`
    case 'two': return `/* readTwo */ SELECT 2`
    default: return impossible(type)
  }
}
function missing(type) {
  switch (type) {
    case 'one': return `SELECT 1`
    case 'two': return `SELECT 2`
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
function incomplete(type) {
  switch (type) {
    case 'one': return `/* incomplete */ SELECT 1`
  }
}
function opaqueDefault(type) {
  switch (type) {
    case 'one': return `/* opaque */ SELECT 1`
    default: return opaque(type)
  }
}
function mutableBuilder(type) {
  switch (type) {
    case 'one': return sql`/* builder */ SELECT 1`
    default: return sql`/* builderOther */ SELECT 2`
  }
}
function interpolation(type) {
  switch (type) {
    case 'one': return `/* interpolated */ SELECT ${type}`
    default: return impossible(type)
  }
}
function defaulted(type = opaque()) {
  switch (type) {
    case 'one': return `/* defaulted */ SELECT 1`
    default: return impossible(type)
  }
}
function changed(type) {
  switch (type) {
    case 'one': return `/* changed */ SELECT 1`
    default: return impossible(type)
  }
}
changed = opaque

function accepted(type) {
  const a = named(type)
  read(a, [1]) // low: true
  const b = missing(type)
  read(b, [1]) // low: false
}
function rejected(type) {
  const a = mixed(type)
  read(a) // low: false
  const b = incomplete(type)
  read(b) // low: false
  const c = opaqueDefault(type)
  read(c) // low: false
  const d = mutableBuilder(type)
  opaque(d)
  read(d) // low: false
  const e = interpolation(type)
  read(e) // low: false
  const f = defaulted(type)
  read(f) // low: false
  const g = named(type)
  opaque(g)
  read(g) // low: false
  let h = named(type)
  h = '/* rewrite */ SELECT 1'
  read(h) // low: false
  const i = named(type)
  opaque(() => i)
  read(i) // low: false
  const j = named(type)
  read(j) // low: false
  opaque(j)
  const k = named(type)
  const alias = k
  read(k) // low: false
  const l = changed(type)
  read(l) // low: false
}
function shadowed(type) {
  const named = opaque
  const query = named(type)
  read(query) // low: false
}
