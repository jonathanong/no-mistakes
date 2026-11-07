import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function workspace() { return { root: mkdtempSync(join(tmpdir(), 'case-')) } }
workspace = () => ({ root: process.cwd() })
const { root } = workspace()
readdirSync(root)
