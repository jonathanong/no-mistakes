import { readdirSync } from 'node:fs'
function list(root = 'src') {
  return readdirSync(root)
}
list()
