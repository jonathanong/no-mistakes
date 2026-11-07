import { readdirSync } from 'node:fs'
import { join } from 'node:path'
export function list(root: string): void {
 function visit(relative: string): void {
  for (const entry of readdirSync(join(root, relative), { withFileTypes: true })) {
   if (entry.isDirectory()) visit(join(relative, entry.name))
  }
 }
 visit('')
}
