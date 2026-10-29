import sql from 'sql-template-strings'
import { read } from '@bench/db'

// Bind holes keep the older mutable-literal proof out of these controls.

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
  if (flag) { noElse = sql`/* one arm */ SELECT ${flag}` }
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
  if (flag) { extra = sql`/* extra */ SELECT ${flag}` } else { extra = sql`/* extra */ SELECT ${flag}` }
  extra = sql`/* changed */ SELECT ${flag}`
  await read(extra) // low: false
  let alias
  if (flag) { alias = sql`/* alias */ SELECT ${flag}` } else { alias = sql`/* alias */ SELECT ${flag}` }
  const escaped = alias
  await read(alias) // low: false
  opaque(escaped)
  let captured
  if (flag) { captured = sql`/* capture */ SELECT ${flag}` } else { captured = sql`/* capture */ SELECT ${flag}` }
  schedule(() => opaque(captured))
  await read(captured) // low: false
  let getter
  if (flag) { getter = sql`/* getter */ SELECT ${flag}` } else { getter = sql`/* getter */ SELECT ${flag}` }
  const holder = { get value() { return getter } }
  opaque(holder.value)
  await read(getter) // low: false
  let condition
  if (condition) { condition = sql`/* condition */ SELECT ${flag}` } else { condition = sql`/* condition */ SELECT ${flag}` }
  await read(condition) // low: false
}

async function shapeControls(flag) {
  let nested
  if (flag) { opaque(1); nested = sql`/* nested */ SELECT ${flag}` } else { nested = sql`/* nested */ SELECT ${flag}` }
  await read(nested) // low: false
  let looped
  while (flag) { if (flag) { looped = sql`/* loop */ SELECT ${flag}` } else { looped = sql`/* loop */ SELECT ${flag}` }; break }
  await read(looped) // low: false
  let finalized
  try { if (flag) { finalized = sql`/* finally */ SELECT ${flag}` } else { finalized = sql`/* finally */ SELECT ${flag}` } } finally { opaque(1) }
  await read(finalized) // low: false
  let used
  if (flag) { opaque(used = sql`/* used */ SELECT ${flag}`) } else { used = sql`/* used */ SELECT ${flag}` }
  await read(used) // low: false
}
