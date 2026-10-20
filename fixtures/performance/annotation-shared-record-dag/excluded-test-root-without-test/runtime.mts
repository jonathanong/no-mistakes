import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'
read(sql`/* runtime */ SELECT 1`)
