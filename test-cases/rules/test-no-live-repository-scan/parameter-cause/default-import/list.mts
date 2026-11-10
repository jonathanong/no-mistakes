import { readdirSync } from 'node:fs'
export default function list(root: string) {
  return readdirSync(root)
}
