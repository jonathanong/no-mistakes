import { read, write as save } from '@example/db'
import { other } from '@other/db'

export function assigned(primary: boolean) {
  const runQuery = primary ? save : read
  const alias = runQuery
  return alias('/* assigned */ SELECT id FROM items')
}
export function direct(primary: boolean) {
  return (primary ? save : read)('/* direct */ SELECT id FROM items')
}
export function nested(primary: boolean, secondary: boolean) {
  const runQuery = primary ? (secondary ? read : save) : read
  return runQuery('/* nested */ SELECT id FROM items')
}
// Each negative keeps an executor-looking name without proving its binding.
export function mixed(primary: boolean) {
  const runQuery = primary ? save : other
  return runQuery('/* mixed */ SELECT id FROM items')
}
export function parameter(primary: boolean, read: unknown) {
  const runQuery = primary ? save : read
  return runQuery('/* parameter */ SELECT id FROM items')
}
export function shadow(primary: boolean) {
  const runQuery = primary ? save : read
  {
    const runQuery = other
    runQuery('/* shadow */ SELECT id FROM items')
  }
  return runQuery('/* outer */ SELECT id FROM items')
}
export function mutable(primary: boolean) {
  let runQuery = primary ? save : read
  runQuery = other
  return runQuery('/* mutable */ SELECT id FROM items')
}
export function beforeDeclaration(primary: boolean) {
  runQuery('/* tdz */ SELECT id FROM items')
  const runQuery = primary ? save : read
}
export function branchShadow(primary: boolean) {
  const runQuery = primary ? save : read
  const read = other
  return runQuery('/* branch-shadow */ SELECT id FROM items')
}

// Conditional syntax is acyclic; nesting must not impose an alias-depth cutoff.
export function deep(primary: boolean) {
  const runQuery = primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (primary ? save : (read))))))))))))
  return runQuery('/* deep */ SELECT id FROM items')
}
export function directMixed(primary: boolean) {
  return (primary ? save : other)('/* direct-mixed */ SELECT id FROM items')
}
export function directShadow(primary: boolean, read: unknown) {
  return (primary ? save : read)('/* direct-shadow */ SELECT id FROM items')
}
export function rebound(primary: boolean) {
  const runQuery = primary ? save : read
  // Recovery syntax must not preserve a proof after an assignment to const.
  runQuery = other
  return runQuery('/* rebound */ SELECT id FROM items')
}
// A class expression's private name shadows the imported executor in methods.
export const Named = class read {
  method(primary: boolean) {
    return (primary ? save : read)('/* class-shadow */ SELECT id FROM items')
  }
}
export function classComputed(primary: boolean) {
  return class read {
    [(primary ? save : read)('/* class-key-shadow */ SELECT id FROM items')]() {}
  }
}
export function classHeritage(primary: boolean) {
  return class read extends ((primary ? save : read)('/* class-extends-shadow */ SELECT id FROM items')) {}
}
export function outsideClass(primary: boolean) {
  const runQuery = primary ? save : read
  return runQuery('/* outside-class */ SELECT id FROM items')
}
