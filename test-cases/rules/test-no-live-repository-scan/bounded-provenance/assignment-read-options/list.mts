import { readdirSync } from 'node:fs'
type Options = { root: string }
export function defaultRead(options: Options) {
  let selected: string
  ;({ root: selected = options.root } = {})
  return readdirSync(options.root)
}
export function computedRead(options: Options) {
  let selected: string
  ;({ [options.root]: selected } = {})
  return readdirSync(options.root)
}
export function nestedDefaultRead(options: Options) {
  let selected: string
  ;({ nested: { root: selected = options.root } } = { nested: {} })
  return readdirSync(options.root)
}
export function nestedComputedRead(options: Options) {
  let selected: string
  ;([{ [options.root]: selected }] = [{}])
  return readdirSync(options.root)
}
export function assignmentRead(options: Options) {
  let selected: string
  selected = options.root
  return readdirSync(options.root)
}
export function forOfRead(options: Options) {
  for (const value of options.root) {}
  return readdirSync(options.root)
}
