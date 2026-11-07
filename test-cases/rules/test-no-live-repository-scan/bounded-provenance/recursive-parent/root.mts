import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(root: string): void {
 readdirSync(root)
 list(join(root, '..'))
}
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
