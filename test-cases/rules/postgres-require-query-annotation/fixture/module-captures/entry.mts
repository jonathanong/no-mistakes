import { read as execute } from '@example/db'
import { read, replace } from './shared.mts'

function discard(value: unknown) { return 0 }

// In ordinary mode shared is lazy. The first arm creates a dead local frame
// before reading its imported function; compaction must retain/remap the module.
export function choose(flag: boolean) {
  return flag ? (discard('dead-before-module'), read) : (replace(), read)
}

// Both branch-returned functions refer to the same registered module frame.
export function executeChoice(flag: boolean) {
  return execute(choose(flag)())
}

export async function dynamicImportThenRead() {
  await import('./lazy-mutator.mts')
  return execute(read())
}

// This closure has the same source owner as a module function but captures a
// local invocation frame. Checking the path alone would certify the wrong SQL.
export function capturedLocalOwner() {
  const statement = 'SELECT local_without_annotation'
  const callback = () => statement
  return execute(callback())
}
