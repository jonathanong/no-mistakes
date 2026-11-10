import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import * as mod from './list.mts'
const root = mkdtempSync(join(tmpdir(), 'case-'))
mod.list(root)
