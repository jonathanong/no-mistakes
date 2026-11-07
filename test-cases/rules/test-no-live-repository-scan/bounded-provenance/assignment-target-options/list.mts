import { readdirSync } from 'node:fs'
type Options = { root: string }
export function objectTarget(options: Options) {
  ;({ root: options.root } = { root: process.cwd() })
  return readdirSync(options.root)
}
export function nestedObjectTarget(options: Options) {
  ;({ nested: { root: options.root } } = { nested: { root: process.cwd() } })
  return readdirSync(options.root)
}
export function nestedArrayTarget(options: Options) {
  ;([[options.root]] = [[process.cwd()]])
  return readdirSync(options.root)
}
export function objectRestTarget(options: Options) {
  ;({ ...options.root } = { root: process.cwd() })
  return readdirSync(options.root)
}
export function arrayRestTarget(options: Options) {
  ;([...options.root] = [process.cwd()])
  return readdirSync(options.root)
}
export function defaultTarget(options: Options) {
  ;({ root: options.root = process.cwd() } = {})
  return readdirSync(options.root)
}
export function nestedDefaultTarget(options: Options) {
  ;({ nested: [options.root = process.cwd()] } = { nested: [] })
  return readdirSync(options.root)
}
export function forOfTarget(options: Options) {
  for ({ root: options.root } of [{ root: process.cwd() }]) {}
  return readdirSync(options.root)
}
export function forInTarget(options: Options) {
  for (options.root in {}) {}
  return readdirSync(options.root)
}
export function castTarget(options: Options) {
  ;({ root: (options.root as string) } = { root: process.cwd() })
  return readdirSync(options.root)
}
