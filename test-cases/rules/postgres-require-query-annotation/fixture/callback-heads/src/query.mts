import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { page, views, mixed, escaped, aliased, replaced, prototype } from './helpers.mts'

page(sql`/* page */ SELECT 1`, statement => read(statement)) // accepted
views(statement => read(statement)) // accepted
page(sql`SELECT 1`, statement => read(statement)) // unknown
mixed(sql`/* mixed */ SELECT 1`, statement => read(statement)) // unknown
escaped(sql`/* escaped */ SELECT 1`, statement => read(statement)) // unknown
aliased(sql`/* aliased */ SELECT 1`, statement => read(statement)) // unknown
replaced(sql`/* replaced */ SELECT 1`, statement => read(statement)) // unknown
prototype(sql`/* prototype */ SELECT 1`, statement => read(statement)) // unknown
page(sql`/* spread */ SELECT 1`, (...statements) => read(statements[0])) // unknown
page(sql`/* default */ SELECT 1`, (statement = 'SELECT 1') => read(statement)) // unknown
page(sql`/* rewritten */ SELECT 1`, statement => read('SELECT 1')) // missing
