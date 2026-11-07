import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(root: string) { return readdirSync(root) }
function inspect(root: string) { return list(root) }
const root = mkdtempSync(join(tmpdir(), 'case-'))
inspect(root)
