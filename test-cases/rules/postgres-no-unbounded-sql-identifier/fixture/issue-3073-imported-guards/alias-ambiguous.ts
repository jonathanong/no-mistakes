import { SHARED_TABLES as selected } from './alias-ambiguous-a.ts'
import { SHARED_TABLES as other } from './alias-ambiguous-b.ts'
const alias = selected as unknown as string[]
alias.push(process.env.TABLE!)
void other.length
