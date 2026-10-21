import sql from 'sql-template-strings'
// Keep this control free of opaque calls that invalidate builders after the arms.
const query = sql`/* precise */ SELECT 1`
const joined = unknownCondition ? query : query
