import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
readdirSync(join(mkdtempSync(join(tmpdir(), 'case-')), '..'))
