import sql from 'sql-template-strings'
import { read } from '@bench/db'

async function positive(flag, value) {
  let q
  if (flag) { q = sql`/* two arm */ SELECT ${value}` } else { q = sql`/* two arm */ SELECT ${value}` }
  const { rows } = await read(q) // low: true
  let different
  if (flag) { different = sql`/* first name */ SELECT ${value}` } else { different = sql`/* second name */ SELECT ${value}` }
  await read(different) // low: true
}

async function missing(flag) {
  let q
  if (flag) { q = sql`SELECT ${flag}` } else { q = sql`SELECT ${flag}` }
  await read(q) // low: false
  let mixed
  if (flag) { mixed = sql`/* named */ SELECT ${flag}` } else { mixed = sql`SELECT ${flag}` }
  await read(mixed) // low: false
  let noElse
  if (flag) { noElse = sql`/* one arm */ SELECT 1` }
  await read(noElse) // low: false
}

function custom(strings, ...values) { return { strings, values } }
async function controls(flag, value) {
  let q
  if (flag) { q = custom`/* custom */ SELECT ${value}` } else { q = custom`/* custom */ SELECT ${value}` }
  await read(q) // low: false
  let shadowed
  function inner(sql) {
    if (flag) { shadowed = sql`/* shadowed */ SELECT ${value}` } else { shadowed = sql`/* shadowed */ SELECT ${value}` }
    return read(shadowed) // low: false
  }
  await inner(custom)
  let extra
  if (flag) { extra = sql`/* extra */ SELECT 1` } else { extra = sql`/* extra */ SELECT 1` }
  extra = sql`/* changed */ SELECT 1`
  await read(extra) // low: false
  let alias
  if (flag) { alias = sql`/* alias */ SELECT 1` } else { alias = sql`/* alias */ SELECT 1` }
  const escaped = alias
  await read(alias) // low: false
  opaque(escaped)
  let captured
  if (flag) { captured = sql`/* capture */ SELECT 1` } else { captured = sql`/* capture */ SELECT 1` }
  schedule(() => opaque(captured))
  await read(captured) // low: false
  let getter
  if (flag) { getter = sql`/* getter */ SELECT 1` } else { getter = sql`/* getter */ SELECT 1` }
  const holder = { get value() { return getter } }
  opaque(holder.value)
  await read(getter) // low: false
  let condition
  if (condition) { condition = sql`/* condition */ SELECT 1` } else { condition = sql`/* condition */ SELECT 1` }
  await read(condition) // low: false
}

async function shapeControls(flag) {
  let nested
  if (flag) { opaque(1); nested = sql`/* nested */ SELECT 1` } else { nested = sql`/* nested */ SELECT 1` }
  await read(nested) // low: false
  let looped
  while (flag) { if (flag) { looped = sql`/* loop */ SELECT 1` } else { looped = sql`/* loop */ SELECT 1` }; break }
  await read(looped) // low: false
  let finalized
  try { if (flag) { finalized = sql`/* finally */ SELECT 1` } else { finalized = sql`/* finally */ SELECT 1` } } finally { opaque(1) }
  await read(finalized) // low: false
  let used
  if (flag) { opaque(used = sql`/* used */ SELECT 1`) } else { used = sql`/* used */ SELECT 1` }
  await read(used) // low: false
}
