import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { list } from './index.mts'
import * as helpers from './index.mts'
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
consume(helpers)
