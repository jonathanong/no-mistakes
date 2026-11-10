import { readdirSync } from 'node:fs'
function list(root: string) {
  return readdirSync(root)
}
const dir = process.argv[2]
list(dir)
