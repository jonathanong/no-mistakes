import { readdirSync } from 'node:fs'
export function list(root: string) {
  return readdirSync(root)
}
