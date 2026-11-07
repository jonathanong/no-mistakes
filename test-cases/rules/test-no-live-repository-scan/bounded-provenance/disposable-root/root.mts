import { mkdtempDisposableSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
using root = mkdtempDisposableSync(join(tmpdir(), 'case-'))
readdirSync(root.path)
