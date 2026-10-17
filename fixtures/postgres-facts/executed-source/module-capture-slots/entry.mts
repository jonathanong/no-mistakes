import { read as execute } from '@example/db'
import { read, replaceWithAnnotated, replaceWithUnannotated } from './shared.mts'

function discard(value: unknown) { return 0 }

export function directAnnotated() {
  return execute(read())
}

export function directMissing() {
  replaceWithUnannotated()
  return execute(read())
}

export function branchAnnotated(flag: boolean) {
  // Both arms retain the same lazy module function and write the same SQL.
  // The discarded first-arm call frame precedes the lazy module frame.
  const callback = flag
    ? (discard('dead-before-module'), read, replaceWithAnnotated(), read)
    : (replaceWithAnnotated(), read)
  return execute(read())
}

export function branchMissing(flag: boolean) {
  const callback = flag
    ? (discard('dead-before-module'), read, replaceWithUnannotated(), read)
    : (replaceWithUnannotated(), read)
  return execute(read())
}

export function samePathLocal() {
  // The owner path also contains module functions; this capture stays lexical.
  const statements = ['SELECT local_without_annotation']
  const callback = () => statements[0]
  return execute(callback())
}
