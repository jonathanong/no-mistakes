import { read } from '@example/db'

export function conditional() {
  if (true) { var query = `SELECT id FROM posts` }
  return read(query) // finding:conditional
}
export function annotated() {
  if (true) { var query = `/* getPost */ SELECT id FROM posts` }
  return read(query) // known:annotated
}
export function nested() {
  if (true) { { var query = `SELECT id FROM nested_posts` } }
  return read(query) // finding:nested
}
export function sibling() {
  return read(query) // unknown:sibling
}
export function nestedFunction() {
  var query = `SELECT id FROM outer_posts`
  function inner() {
    if (true) { var query = `/* inner */ SELECT id FROM inner_posts` }
    read(query) // known:inner
  }
  inner()
  return read(query) // finding:outer
}
export function shadowing() {
  var query = `SELECT id FROM outer_shadow`
  { let query = `/* inner */ SELECT id FROM let_shadow`; read(query) } // known:let-shadow
  { const query = `/* inner */ SELECT id FROM const_shadow`; read(query) } // known:const-shadow
  return read(query) // finding:shadowing
}
export function letIsolation() {
  if (true) { let query = `SELECT id FROM let_posts` }
  return read(query) // unknown:let-isolation
}
export function constIsolation() {
  if (true) { const query = `SELECT id FROM const_posts` }
  return read(query) // unknown:const-isolation
}
export function classicFor() {
  { for (var query = `SELECT id FROM loop_posts`; false;) {} }
  return read(query) // finding:classic-for
}
export function reassigned() {
  if (true) { var query = `SELECT id FROM reassigned_posts` }
  query = unknownValue
  return read(query) // unknown:reassigned
}
export function conflicting(flag: boolean) {
  if (flag) { var query = `SELECT id FROM conflict_posts` }
  else { var query = `/* conflicting */ SELECT id FROM conflict_posts` }
  return read(query) // unknown:conflicting
}
export function uninitializedRedeclaration() {
  if (true) { var query = `SELECT id FROM redeclared_posts` }
  { var query }
  return read(query) // finding:redeclaration
}
const globalQuery = `SELECT id FROM top_level_posts`
export function beforeDeclaration() {
  read(globalQuery) // unknown:before-declaration
  if (true) { var globalQuery = `/* hoisted */ SELECT id FROM hoisted_posts` }
  read(globalQuery) // known:after-declaration
}
export const arrow = () => {
  if (true) { var query = `SELECT id FROM arrow_posts` }
  return read(query) // finding:arrow
}
export function destructuredShadow() {
  if (true) { var { query } = unknownValue }
  return read(query) // unknown:destructured
}
export function nestedBoundary() {
  const inner = () => { if (true) { var globalQuery = `/* inner */ SELECT id FROM arrow_inner` } }
  function declared() { if (true) { var globalQuery = `/* inner */ SELECT id FROM fn_inner` } }
  class Holder { static { var globalQuery = `/* inner */ SELECT id FROM static_inner`; read(globalQuery) } } // known:static-inner
  return read(globalQuery) // finding:nested-boundary
}
if (true) { var topQuery = `SELECT id FROM top_posts` }
read(topQuery) // finding:top-level
export function direct() {
  return read(`SELECT id FROM posts`) // finding:direct
}
export function bareConditional() {
  if (true) var query = `SELECT id FROM bare_posts`
  return read(query) // finding:bare-conditional
}
export function destructuredReassignment() {
  var query = `SELECT id FROM destructured_posts`
  { var { query } = unknownValue }
  return read(query) // unknown:destructured-reassignment
}
export function iterationReassignment() {
  var query = `SELECT id FROM iterated_posts`
  for (var query of unknownValue) {}
  return read(query) // unknown:iteration-reassignment
}
export function catchShadow() {
  try { throw unknownValue }
  catch (query) {
    var query = `/* catch */ SELECT id FROM catch_posts`
    read(query) // known:catch
  }
  return read(query) // unknown:catch-outer
}
