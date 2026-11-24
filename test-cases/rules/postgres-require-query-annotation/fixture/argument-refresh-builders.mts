import sql from 'sql-template-strings'

const invalidated = sql`/* fixture/invalidated */ SELECT 1`
const retained = sql`/* fixture/retained */ SELECT 2`
const argument = { nested: { invalidated }, retained }
