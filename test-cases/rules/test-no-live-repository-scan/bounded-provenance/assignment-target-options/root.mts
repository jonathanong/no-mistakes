import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { objectTarget, nestedObjectTarget, nestedArrayTarget, objectRestTarget, arrayRestTarget, defaultTarget, nestedDefaultTarget, forOfTarget, forInTarget, castTarget } from './list.mts'
const root = mkdtempSync(join(tmpdir(), 'case-'))
objectTarget({ root })
nestedObjectTarget({ root })
nestedArrayTarget({ root })
objectRestTarget({ root })
arrayRestTarget({ root })
defaultTarget({ root })
nestedDefaultTarget({ root })
forOfTarget({ root })
forInTarget({ root })
castTarget({ root })
