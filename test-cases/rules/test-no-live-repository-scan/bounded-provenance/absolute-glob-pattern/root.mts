import { mkdtempSync, globSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(pattern: string) {
 const cwd = mkdtempSync(join(tmpdir(), 'case-'))
 return globSync(pattern, { cwd })
}
list('/repository/**/*.mts')
