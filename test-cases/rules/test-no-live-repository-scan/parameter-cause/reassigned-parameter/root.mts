import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(root: string) {
  root = 'src'
  return readdirSync(root)
}
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
