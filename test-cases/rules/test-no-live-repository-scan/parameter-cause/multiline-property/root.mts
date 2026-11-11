import { readdirSync } from 'node:fs'

function shorthand(options: { root: string; include: string[] }) {
  return readdirSync(options.root)
}
function renamed(options: { directory: string }) {
  return readdirSync(options.directory)
}
function positional(root: string) {
  return readdirSync(root)
}
function bounded(options: { root: URL }) {
  return readdirSync(options.root)
}

const root = process.argv[2]
const directory = process.argv[3]
shorthand({
  include: ['first-long-pattern-to-push-the-root-outside-the-outer-argument-excerpt', 'another'],
  root,
})
renamed({
  directory:
    directory,
})
positional(
  directory,
)
bounded({
  root: new URL('./fixtures', import.meta.url),
})
