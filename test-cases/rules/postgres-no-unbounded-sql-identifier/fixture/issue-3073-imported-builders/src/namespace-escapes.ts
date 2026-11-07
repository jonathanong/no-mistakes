import { bare, alias, quoted, opaque, unrelated, 'statement.query' as quotedAlias } from './namespace-escape-barrel'
declare function escape(value: unknown): void

escape(bare)
const held = alias
held.statement.text = process.env.TABLE
quoted['quoted.name'].text = process.env.TABLE
opaque[process.env.FIELD!].text = process.env.TABLE
quotedAlias.text = process.env.TABLE
escape(unrelated)
