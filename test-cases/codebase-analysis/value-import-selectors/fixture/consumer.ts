import { TABLES as selected, statement, run, mutable, options as unused } from '@catalog/source'
import { TABLES as open, otherHelper } from './other.ts'
import { RENAMED as alias } from './alias.ts'
import * as namespace from './barrel.ts'
import { catalog as grouped } from './group.ts'
import * as nested from './group.ts'
import fallback from './source.ts'
import { TABLES as ambiguous } from './ambiguous.ts'
import { TABLES as unknown } from './unknown-star.ts'
import type { Only } from './source.ts'
import { options as shadowed } from './source.ts'

void selected.length
void open
void statement.text
run()
otherHelper()
void alias.length
void namespace.TABLES.length
void namespace.statement.text
void grouped.TABLES.length
void nested.catalog.TABLES.length
void fallback.table
void mutable
void ambiguous.length
void unknown.length
type Erased = typeof unused
export function shadow(shadowed: { table: string }) { return shadowed.table }
