import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { defaultRead, computedRead, nestedDefaultRead, nestedComputedRead, assignmentRead, forOfRead } from './list.mts'
const root = mkdtempSync(join(tmpdir(), 'case-'))
defaultRead({ root })
computedRead({ root })
nestedDefaultRead({ root })
nestedComputedRead({ root })
assignmentRead({ root })
forOfRead({ root })
