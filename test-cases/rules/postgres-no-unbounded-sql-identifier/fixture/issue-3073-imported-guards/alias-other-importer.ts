import { OTHER_ALIAS_TABLES as tables } from './alias-tables.ts'
const alias = tables as unknown as string[]
alias.push(process.env.TABLE!)
