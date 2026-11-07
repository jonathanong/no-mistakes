import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(root: string): void {
 for (const entry of readdirSync(root, { withFileTypes: true })) {
  if (entry.isDirectory()) list(join(root, entry.name))
 }
}
function unsafe(root: string) { list(root) }
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
unsafe(process.argv[2])
