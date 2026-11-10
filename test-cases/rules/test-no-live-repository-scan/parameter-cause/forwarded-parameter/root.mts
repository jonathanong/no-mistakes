import { readdirSync } from 'node:fs'
export function list(root: string): void {
  readdirSync(root)
  list(root)
}
