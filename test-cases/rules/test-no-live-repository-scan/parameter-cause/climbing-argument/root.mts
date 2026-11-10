import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(root: string) {
  return readdirSync(root)
}
list(join(mkdtempSync(join(tmpdir(), 'case-')), '..'))
