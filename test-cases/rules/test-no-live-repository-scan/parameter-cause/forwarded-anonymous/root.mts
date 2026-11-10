import { readdirSync } from 'node:fs'
function list(root: string) {
  return readdirSync(root)
}
['a'].forEach((root: string) => list(root))
