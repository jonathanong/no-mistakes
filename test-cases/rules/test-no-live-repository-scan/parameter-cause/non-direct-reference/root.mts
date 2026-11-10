import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(root: string) {
  return readdirSync(root)
}
const alias = list
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
void alias
