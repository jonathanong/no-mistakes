import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function workspace() { return mkdtempSync(join(tmpdir(), 'case-')) }
const root = workspace()
readdirSync(root)
