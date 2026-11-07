import * as tables from './alias-tables.ts'
const alias = tables.NAMESPACE_ALIAS_TABLES as unknown as string[]
alias.push(process.env.TABLE!)
