import { read } from '@bench/db'
import { initializerAwait, taggedAwait, disposal, getterEscape, secondPosition, named, namedAlias, missing, early, finallyOverride, postAwait, escape, unresolved, cyclic, alias, captured, customTag, begin, comment, reassigned, defaulted, deep, budget } from './helpers.mjs'
import * as ns from './helpers.mjs'

export async function run(user, options) {
  const positive0 = await named(user, options)
  await read(positive0) // Literal
  const positive1 = await namedAlias(user, options)
  await read(positive1) // Literal
  const positive2 = await missing()
  await read(positive2) // Missing
  const negative0 = await early()
  await read(negative0) // Unproven
  const negative1 = await finallyOverride()
  await read(negative1) // Unproven
  const negative2 = await postAwait()
  await read(negative2) // Unproven
  const negative3 = await escape()
  await read(negative3) // Unproven
  const negative4 = await unresolved()
  await read(negative4) // Unproven
  const negative5 = await cyclic()
  await read(negative5) // Unproven
  const negative6 = await alias()
  await read(negative6) // Unproven
  const negative7 = await captured()
  await read(negative7) // Unproven
  const negative8 = await customTag()
  await read(negative8) // Unproven
  const negative9 = await begin()
  await read(negative9) // Unproven
  const negative10 = await comment()
  await read(negative10) // Unproven
  const negative12 = await reassigned()
  await read(negative12) // Unproven
  const negative13 = await defaulted()
  await read(negative13) // Unproven
  const negative14 = await deep()
  await read(negative14) // Unproven
  const negative15 = await budget()
  await read(negative15) // Unproven

  const getter = await getterEscape()
  await read(getter) // Unproven
  const position = await secondPosition()
  await read(position) // Unproven
  const resource = await disposal()
  await read(resource) // Unproven
  const initializer = await initializerAwait()
  await read(initializer) // Unproven
  const tagged = await taggedAwait()
  await read(tagged) // Unproven
  const noAwait = named(user, options)
  await read(noAwait) // Unproven
  const namespaced = await ns.named(user, options)
  await read(namespaced) // Unproven
  const optional = await named?.(user, options)
  await read(optional) // Unproven
  const spread = await named(...args)
  await read(spread) // Unproven
  const chained = await named(user, options).then(identity)
  await read(chained) // Unproven
  const extraRead = await named(user, options)
  await read(extraRead) // Unproven
  observe(extraRead)
  const intervening = await named(user, options)
  opaque()
  await read(intervening) // Unproven
  const changed = await named(user, options)
  await read(changed) // Unproven
  changed = other()
  const capture = await named(user, options)
  await read(capture) // Unproven
  register(() => capture)
}

export async function shadow(named) {
  const query = await named()
  read(query) // Unproven
}
