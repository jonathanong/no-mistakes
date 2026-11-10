import { readdirSync } from 'node:fs'
function list(root: string) {
  return readdirSync(root)
}
list(process.cwd())
