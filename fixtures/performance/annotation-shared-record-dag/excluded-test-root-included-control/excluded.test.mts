// Deliberately unannotated: excluded from findings, but still projected.
import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
read(sql`SELECT 2`)
