import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
const list = function (root: string) {
  return readdirSync(root)
}
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
