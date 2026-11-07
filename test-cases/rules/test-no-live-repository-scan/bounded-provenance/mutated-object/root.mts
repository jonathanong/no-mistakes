import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
const root = mkdtempSync(join(tmpdir(), 'case-'))
const options = { root }
options.root = process.cwd()
readdirSync(options.root)
