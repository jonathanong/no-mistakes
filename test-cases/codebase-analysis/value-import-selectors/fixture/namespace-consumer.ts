import { grouped, imported, relayed } from './namespace-exports.ts'
import { grouped as throughStar } from './namespace-star.ts'
import { imported as explicitValue } from './namespace-ambiguous.ts'
import { grouped as unknown } from './namespace-unknown.ts'
import { grouped as collided } from './namespace-collision.ts'
import * as quoted from './namespace-quoted-star.ts'
import * as direct from './source.ts'
import * as nested from './namespace-star.ts'
import * as unrelated from './namespace-unrelated.ts'
import type * as erased from './source.ts'

escape(grouped)
const alias = imported
alias.statement.text = 'open'
void relayed.statement.text
escape(throughStar)
void explicitValue
escape(unknown)
escape(collided)
escape(quoted)
escape(direct)
escape(nested)
escape(unrelated)
type Erased = typeof erased
void nested.grouped.statement.text
