export const PRODUCER_ALIAS_TABLES = ['alpha'] as const
const alias = PRODUCER_ALIAS_TABLES as unknown as string[]
alias.push(process.env.TABLE!)
