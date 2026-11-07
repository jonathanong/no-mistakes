import { readdirSync } from 'node:fs'
import { join } from 'node:path'
export function list(root: string): void {
 for (const entry of readdirSync(root, { withFileTypes: true })) {
  if (entry.isDirectory()) list(join(root, entry.name))
 }
}
